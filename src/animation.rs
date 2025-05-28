use chrono::Utc;
use include_dir::{include_dir, Dir};
use ratatui::{layout::Alignment, widgets::{Block, Borders, Paragraph}};

static TALKING_FRAMES_DIR: Dir = include_dir!("src/frames/talking");

fn get_frames() -> Vec<&'static str> {
    TALKING_FRAMES_DIR.files()
        .filter_map(|f| f.contents_utf8())
        .collect()
}

pub struct Animation {
    talking_frames: Vec<&'static str>,
    frames: Vec<&'static str>,
    current_frame: usize,
    last_time: i64,
}

impl Animation {
    pub fn new() -> Self {
        Self {
            talking_frames: get_frames(),
            frames: get_frames(),
            current_frame: 0,
            last_time: Utc::now().timestamp_millis(),
        }
    }

    pub fn ascii_art_widget(&mut self, box_width: usize) -> Paragraph {
        let current_frame = &self.frames[self.current_frame];
        let padded_frame = Self::pad_ascii_frame(current_frame, box_width);

        if Utc::now().timestamp_millis() - self.last_time > 200 {
            self.current_frame = (self.current_frame + 1) % self.frames.len();
            self.last_time = Utc::now().timestamp_millis();
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
}