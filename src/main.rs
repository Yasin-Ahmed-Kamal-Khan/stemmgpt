/// A Ratatui example that demonstrates how to draw on a canvas.
///
/// This example demonstrates how to draw various shapes such as rectangles, circles, and lines
/// on a canvas. It also demonstrates how to draw a map.
///
/// This example runs with the Ratatui library code in the branch that you are currently
/// reading. See the [`latest`] branch for the code which works with the most recent Ratatui
/// release.
///
/// [`latest`]: https://github.com/ratatui/ratatui/tree/latest


use std::{
    time::{Duration, Instant}
};
use std::io;
use ratatui::{layout::Alignment, style::Style, widgets::{Borders, Paragraph, Wrap}, Frame};
use color_eyre::Result;
use crossterm::event::{
    self, Event, KeyCode, KeyEventKind,
};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Stylize};
use ratatui::symbols::Marker;
use ratatui::text::Text;
use ratatui::widgets::canvas::{Canvas, Rectangle};
use ratatui::widgets::{Block, Widget};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use include_dir::{include_dir, Dir};
use chrono::Utc;
static FRAMES_DIR: Dir = include_dir!("src/frames/");

fn get_frames() -> Vec<&'static str> {
    FRAMES_DIR.files()
        .filter_map(|f| f.contents_utf8())
        .collect()
}
fn main() -> Result<()> {
    color_eyre::install()?;

    // Enable raw mode for terminal
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let app_result = App::new().run(terminal);

    // Clean shutdown
    crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;

    app_result
}
enum InputMode {
    Normal,
    Editing,
}

struct App {
    exit: bool,
    frames: Vec<&'static str>,
    current_frame: usize,
    last_time: i64,
    marker: Marker,

    /// Current value of the input box
    input: String,
    /// Position of cursor in the editor area.
    character_index: usize,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    messages: Vec<String>,
}

impl App {
    fn new() -> Self {
        Self {
            exit: false,
            marker: Marker::Dot,
            frames: get_frames(),
            current_frame: 0,
            last_time: Utc::now().timestamp_millis(),

            input: String::new(),
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            character_index: 0,
          }
    }

    pub fn run(mut self, mut terminal: Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        let tick_rate = Duration::from_millis(16);
        let mut last_tick = Instant::now();
        while !self.exit {
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
                            // KeyCode::Enter => self.submit_message(),
                            KeyCode::Char(to_insert) => self.enter_char(to_insert),
                            // KeyCode::Backspace => self.delete_char(),
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
        let header = Text::from_iter([
            "Canvas Example".bold(),
            "<q> Quit | <enter> Change Marker | <hjkl> Move".into(),
        ]);


        let base_area = frame.size();

        // 2. Create your main layout divisions
        let main_layout = Layout::vertical([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Content area
            Constraint::Length(1)   // Footer
        ]).split(base_area);  // Split the frame's Rect


        let vertical = Layout::vertical([
            Constraint::Length(header.height() as u16),
            Constraint::Percentage(50),
            Constraint::Percentage(50),
        ]).split(main_layout[1]);

        // Destructure the chunks
        let [_, up, down] = [vertical[0], vertical[1], vertical[2]];

        let horizontal =
            Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]);
        let [input_box, ascii_art_box] = horizontal.areas(up);
        let [_, boxes] = horizontal.areas(down);

        frame.render_widget(App::ascii_art_widget(self, ascii_art_box.width.into()), ascii_art_box);
        frame.render_widget(self.input_canvas(), input_box);
        frame.render_widget(self.boxes_canvas(boxes), boxes);
    }

    fn ascii_art_widget(app: &mut App, box_width: usize) -> Paragraph {
        let current_frame = &app.frames[app.current_frame];
        let padded_frame = Self::pad_ascii_frame(current_frame, box_width);

        if Utc::now().timestamp_millis() - app.last_time > 500 {
            app.current_frame = (app.current_frame + 1) % app.frames.len();
            app.last_time = Utc::now().timestamp_millis();
        }

        Paragraph::new(padded_frame)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("ASCII Art")
            )
            .alignment(Alignment::Left)

    }

    fn pad_ascii_frame(frame: &str, target_width: usize) -> String {
        // Find the longest line in the ASCII frame
        let frame_width = frame.lines()
            .map(|line| line.len())
            .max()
            .unwrap_or(0);

        // Calculate equal left/right padding
        let total_padding = target_width.saturating_sub(frame_width);
        let left_pad = total_padding / 2;
        let right_pad = total_padding - left_pad;

        // Apply padding to each line
        frame.lines()
            .map(|line| {
                format!(
                    "{}{}{}",
                    " ".repeat(left_pad),
                    line,
                    " ".repeat(right_pad)
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn boxes_canvas(&self, area: Rect) -> impl Widget {
        let left = 0.0;
        let right = f64::from(area.width);
        let bottom = 0.0;
        let top = f64::from(area.height).mul_add(2.0, -4.0);
        Canvas::default()
            .block(Block::bordered().title("Rects"))
            .marker(self.marker)
            .x_bounds([left, right])
            .y_bounds([bottom, top])
            .paint(|ctx| {
                for i in 0..=11 {
                    ctx.draw(&Rectangle {
                        x: f64::from(i * i + 3 * i) / 2.0 + 2.0,
                        y: 2.0,
                        width: f64::from(i),
                        height: f64::from(i),
                        color: Color::Red,
                    });
                    ctx.draw(&Rectangle {
                        x: f64::from(i * i + 3 * i) / 2.0 + 2.0,
                        y: 21.0,
                        width: f64::from(i),
                        height: f64::from(i),
                        color: Color::Blue,
                    });
                }
                for i in 0..100 {
                    if i % 10 != 0 {
                        ctx.print(f64::from(i) + 1.0, 0.0, format!("{i}", i = i % 10));
                    }
                    if i % 2 == 0 && i % 10 != 0 {
                        ctx.print(0.0, f64::from(i), format!("{i}", i = i % 10));
                    }
                }
            })
    }

    fn input_canvas(&mut self) -> impl Widget + '_{
        // Create a styled block with title
        let block = Block::bordered()
            .title(" Input ")
            .title_alignment(Alignment::Left)
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            });

        // Create the paragraph with proper cursor handling
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

}