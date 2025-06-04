use std::{
    sync::LazyLock, time::{Duration, Instant}
};
use std::io;
use ratatui::{layout::Alignment, style::Style, widgets::{Borders, Paragraph, Wrap}, Frame};
use color_eyre::{eyre::Ok, owo_colors::colors::Red, Result};
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
use pyo3::prelude::*;
use pyo3::types::PyModule;


use crate::{animation::{Animation, State}, typewriter::Typewriter};

static HEADER_TEXT: LazyLock<Text<'static>> = LazyLock::new(|| {
    Text::from_iter([
        "".fg(Color::Cyan),
        "         🤖 STEMM GPT 🧬 ☪️    ".fg(Color::Green).bold(),
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
                            KeyCode::PageUp => self.animation.set_state(State::DYING),
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

        // Create main layout with header and content
        let main_layout = Layout::vertical([
            Constraint::Length(HEADER_TEXT.height() as u16), // Header
            Constraint::Min(0), // Content area
        ]).split(base_area);

        let [heading, content_area] = [main_layout[0], main_layout[1]];

        // Split content area horizontally: left side for output, right side for input/animation
        let horizontal_layout = Layout::horizontal([
            Constraint::Percentage(50), // Left side - output canvas
            Constraint::Percentage(50), // Right side - input and animation
        ]).split(content_area);

        let [animation_area, right_side] = [horizontal_layout[0], horizontal_layout[1]];

        // Split right side vertically: top for input, bottom for animation
        let right_vertical = Layout::vertical([
            Constraint::Percentage(50), // Input canvas
            Constraint::Percentage(50), // Animation
        ]).split(right_side);

        let [input_area, left_side] = [right_vertical[0], right_vertical[1]];

        // Render all widgets
        frame.render_widget(self.header(), heading);
        frame.render_widget(self.output_canvas(), left_side);
        frame.render_widget(self.input_canvas(), input_area);
        self.animation.render_ascii_art_widget(animation_area, frame);
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
        use std::fs;
        use std::process::Command;
        use std::io::Write;
        // 1. Write input to file
        if let std::result::Result::Ok(mut file) = fs::File::create("input.txt") {
            let _ = file.write_all(self.input.as_bytes());
        }
        // 2. Run ai.py as a process
        let status = Command::new("python")
            .arg("ai.py")
            .status();
        // 3. Read output from file
        let ai_reply = match fs::read_to_string("output.txt") {
            std::result::Result::Ok(contents) => contents,
            std::result::Result::Err(_) => "Error reading AI response".to_string(),
        };
        self.typewriter.add_message(ai_reply);
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