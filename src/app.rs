use std::{
    sync::LazyLock, time::{Duration, Instant}
};
use std::io;
use ratatui::{layout::Alignment, style::Style, widgets::{Borders, Paragraph, Wrap}, Frame};
use color_eyre::{owo_colors::colors::Red, Result};
use crossterm::event::{
    self, Event, KeyCode, KeyEventKind,
};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Color, Stylize};
use ratatui::text::Text;
use ratatui::widgets::{Block, Widget};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use chrono::Utc;

use crate::{animation::{Animation, State}, typewriter::Typewriter};

static HEADER_TEXT: LazyLock<Text<'static>> = LazyLock::new(|| {
    Text::from_iter([
        "".fg(Color::Cyan),
        "         🤖 STEM GPT 🧬      ".fg(Color::Green).bold(),
        "     AI Assistant for STEM   ".fg(Color::Blue),
        "".fg(Color::Cyan),
        "".into(),
        "Press 'e' to edit | 'q' to quit".fg(Color::Yellow).italic(),
    ])
});

enum InputMode {
    Normal,
    Editing,
}

pub struct App {
    exit: bool,
    /// Current value of the input box
    input: String,
    /// Position of cursor in the editor area.
    character_index: usize,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    typewriter: Typewriter,
    animation: Animation,
}

impl App {
    pub fn new() -> Self {
        Self {
            exit: false,
            input: String::new(),
            input_mode: InputMode::Editing,
            character_index: 0,
            typewriter: Typewriter::new(),
            animation: Animation::new(),
          }
    }

    pub fn run(mut self, mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        let tick_rate = Duration::from_millis(16);
        let mut last_tick = Instant::now();
        while !self.exit {
            match self.typewriter.update_typewriter()  {
                Some(state) => self.animation.set_state(state),
                None => {},
            };

            terminal.draw(|frame| self.render(frame))?;
            let timeout = tick_rate.saturating_sub(last_tick.elapsed());
            if !event::poll(timeout)? {
                last_tick = Instant::now();
                continue;
            }
            match event::read()? {
                Event::Mouse(_) => {},
                Event::Key(key) => {
                    match self.input_mode {
                        InputMode::Normal => match key.code {
                            KeyCode::Char('e') => {
                                self.input_mode = InputMode::Editing;
                            }
                            KeyCode::Char('q') => {
                                return Ok(());
                            }
                            _ => {}
                        },
                        InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                            KeyCode::Enter => self.submit_message(),
                            KeyCode::Char(to_insert) => self.enter_char(to_insert),
                            KeyCode::Backspace => self.delete_char(),
                            KeyCode::Left => self.move_cursor_left(),
                            KeyCode::Right => self.move_cursor_right(),
                            KeyCode::Esc => self.input_mode = InputMode::Normal,
                            _ => {}
                        },
                        InputMode::Editing => {}
                    }
                }

                _ => {}
            }
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let base_area = frame.size();

        // 2. Create your main layout divisions
        let main_layout = Layout::vertical([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Content area
            Constraint::Length(1)   // Footer
        ]).split(base_area);  // Split the frame's Rect


        let vertical = Layout::vertical([
            Constraint::Length(HEADER_TEXT.height() as u16),
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]).split(main_layout[1]);

        // Destructure the chunks
        let [heading, up, down] = [vertical[0], vertical[1], vertical[2]];

        let horizontal =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let [input_box, ascii_art_box] = horizontal.areas(up);
        let [_, boxes] = horizontal.areas(down);

        frame.render_widget(self.header(), heading);
        frame.render_widget(self.animation.ascii_art_widget(ascii_art_box.width.into()), ascii_art_box);
        frame.render_widget(self.input_canvas(), input_box);
        frame.render_widget(self.output_canvas(), boxes);
    }

    fn output_canvas(&mut self) -> impl Widget + '_ {
        self.typewriter.output_canvas()
    }

    fn input_canvas(&mut self) -> impl Widget + '_ {
        let block = Block::bordered()
            .title(" Input ")
            .title_alignment(Alignment::Left)
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            });
        Paragraph::new(self.input.as_str())
            .block(block)
            .wrap(Wrap { trim: false })
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn submit_message(&mut self) {
        self.typewriter.add_message(self.input.clone());
        self.input.clear();
        self.reset_cursor();
    }

    const fn reset_cursor(&mut self) {
        self.character_index = 0;
    }

    fn header(&mut self) -> Paragraph<'_> {
        let header = HEADER_TEXT.clone();

        Paragraph::new(header)
            .alignment(Alignment::Center)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Magenta))
                    .title(" Welcome ")
                    .title_alignment(Alignment::Center)
            )
    }
}