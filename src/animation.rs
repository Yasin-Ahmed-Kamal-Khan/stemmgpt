use ratatui::{style::Style, text::Span, widgets::{Borders, Paragraph, Wrap}, Frame};
use rand::Rng;
use chrono::Utc;
use include_dir::{include_dir, Dir};
use ratatui::{layout::{Alignment, Constraint, Direction, Layout, Rect}, style::{palette::material::RED, Color, Stylize}, text::{Line, Text}, widgets::{canvas::Rectangle, Block}};

static DYING_FRAMES_DIR: Dir = include_dir!("src/frames/dying");

fn get_frames(dir: &'static Dir<'static>) -> Vec<&'static str> {
    dir.files()
        .filter_map(|f| f.contents_utf8())
        .collect()
}

pub enum State {
    TALKING,
    IDLE,
    DYING,
}

pub struct Animation {
    last_talking_frame: Vec<String>,
    dying_frames: Vec<&'static str>,
    current_frame: usize,
    last_time: i64,
    state: State,
    talking_frame_num: usize,
    change_talking_frame: bool,
    until_talking: i32,
}

impl Animation {
    pub fn new() -> Self {
        Self {
            last_talking_frame: vec!["".to_string()],
            dying_frames: get_frames(&DYING_FRAMES_DIR),
            current_frame: 0,
            last_time: Utc::now().timestamp_millis(),
            state: State::IDLE,
            talking_frame_num: 0,
            change_talking_frame: false,
            until_talking: 0,
        }
    }

    pub fn render_ascii_art_widget(&mut self, animation_area: Rect, frame: &mut Frame) {
        let box_width = animation_area.width as usize;
        let box_height = animation_area.height as usize;
        let border_block =
            Block::default()
                .borders(Borders::ALL)
                .title("ASCII Art");

        frame.render_widget(border_block.clone(), animation_area);
        let inner_animation_area = border_block.inner(animation_area);


        let (render_area, padded_frame) = match self.state {
            State::TALKING => {
                if Utc::now().timestamp_millis() - self.last_time > 100  {
                    self.last_time = Utc::now().timestamp_millis();
                    self.talking_frame_num += 1;
                    self.change_talking_frame = true;
                }
                if self.change_talking_frame {
                    self.last_talking_frame = self.create_sound_wave(box_width, box_height, self.talking_frame_num);
                    self.change_talking_frame = false;
                }

                let text_lines: Vec<Line> = self.last_talking_frame.clone().into_iter()
                    .map(|line| Line::from(Span::styled(line, Style::default().fg(Color::Yellow))))
                    .collect();
                (inner_animation_area, Text::from(text_lines))
            },
            State::IDLE => {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Percentage(50), // Top half
                        Constraint::Length(1),      // Line (1 row height)
                        Constraint::Percentage(50), // Bottom half
                    ])
                    .split(inner_animation_area);

                // Create the line text
                let line_text = Text::from("█".repeat(chunks[1].width as usize));
                (chunks[1], line_text)
            },
            State::DYING => {
                let current_frame = self.dying_frames[0];
                (inner_animation_area, Text::from(Self::pad_ascii_frame(current_frame, box_width)))
            },
        };

        frame.render_widget(Paragraph::new(padded_frame)
            .alignment(Alignment::Left),
            render_area)

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

    pub fn set_state(&mut self, state: State) {
        match self.state {
            State::DYING => {},
            _ => self.state = state
        }
    }

    fn create_sound_wave(&mut self, width: usize, height: usize, frame: usize) -> Vec<String> {
        let center = height / 2;
        let mut lines = Vec::new();

        if self.until_talking > 0 {
            for y in 0..height {
                let mut line = String::new();
                for _ in 0..width {
                    let target_y = center as i32;
                    let distance = (y as i32 - target_y).abs();
                    let char = match distance {
                        0 => '█',
                        _ => ' ',
                    };
                    line.push(char);
                }
                lines.push(line);
            }
            self.until_talking -= 1;
            return lines;
        }

        let mut rng: rand::prelude::ThreadRng = rand::thread_rng();
        let center = height / 2;
        let mut amplitudes = Vec::new();

        // Control speech-like pauses and bursts
        let is_silent: bool = rng.gen_bool(0.2);
        let burst_strength: f32 = if is_silent {
            self.until_talking = 3;
            0.2
        } else {
            rng.gen_range(0.5..1.5)
        };

        for x in 0..width {
            let t = (x as f32 + frame as f32 * 0.5) * 0.3;

            let wave = (t.sin() * 0.5 + (t * 3.0).sin() * 0.3) * burst_strength;
            let jitter = rng.gen_range(-0.2..0.2);
            let base_amplitude = wave + jitter;

            // Push amplitude (clamped to avoid overflow)
            let amplitude = (base_amplitude * center as f32).clamp(-(center as f32), center as f32);
            amplitudes.push(amplitude as i32);
        }

        // Render lines
        for y in 0..height {
            let mut line = String::new();
            for x in 0..width {
                let target_y = center as i32 + amplitudes[x];
                let distance = (y as i32 - target_y).abs();
                let char = match distance {
                    0 => '█',
                    1 => '▒',
                    _ => ' ',
                };
                line.push(char);
            }
            lines.push(line);
        }
        lines
    }
}
