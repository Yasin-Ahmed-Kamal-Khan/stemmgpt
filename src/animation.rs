use rand::Rng;
use chrono::Utc;
use include_dir::{include_dir, Dir};
use ratatui::{layout::Alignment, style::{palette::material::RED, Color, Stylize}, text::{Line, Text}, widgets::{Block, Borders, Paragraph}};

static TALKING_FRAMES_DIR: Dir = include_dir!("src/frames/talking");
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
    talking_frames: Vec<&'static str>,
    dying_frames: Vec<&'static str>,
    current_frame: usize,
    last_time: i64,
    state: State,
}

impl Animation {
    pub fn new() -> Self {
        Self {
            talking_frames: get_frames(&TALKING_FRAMES_DIR),
            dying_frames: get_frames(&DYING_FRAMES_DIR),
            current_frame: 0,
            last_time: Utc::now().timestamp_millis(),
            state: State::IDLE,
        }
    }

    pub fn ascii_art_widget(&mut self, box_width: usize) -> Paragraph {
        let padded_frame = match self.state {
            State::TALKING => {
                if Utc::now().timestamp_millis() - self.last_time > 200 {
                    self.current_frame = (self.current_frame + 1) % self.talking_frames.len();
                    self.last_time = Utc::now().timestamp_millis();
                }

                let current_frame = &self.talking_frames[self.current_frame];
                Text::from(Self::pad_ascii_frame(current_frame, box_width))
            },
            State::IDLE => {
                Animation::horizontal_line(box_width, &"_".to_string(), Color::Red)
            },
            State::DYING => {
                let current_frame = self.dying_frames[0];
                Text::from(Self::pad_ascii_frame(current_frame, box_width))
            },
        };

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

    fn horizontal_line(width: usize, char: &str, color: Color) -> Text<'static> {
        char.repeat(width).fg(color).into()
    }

    pub fn set_state(&mut self, state: State) {
        match self.state {
            State::DYING => {},
            _ => self.state = state
        }
    }
}
