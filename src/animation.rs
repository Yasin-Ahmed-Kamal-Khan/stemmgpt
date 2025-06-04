use std::{time::Duration, usize::{MAX, MIN}};

use ratatui::{style::Style, text::Span, widgets::{Borders, Paragraph}, Frame};
use rand::{rngs::ThreadRng, Rng};
use chrono::Utc;
use include_dir::{include_dir, Dir};
use ratatui::{layout::{Alignment, Rect}, style::{Color, Stylize}, text::{Line, Text}, widgets::Block};
use rodio::{source::SineWave, OutputStream, Sink, Source};

static DYING_FRAMES_DIR: Dir = include_dir!("src/frames/dying");
static IDLE_FRAMES_DIR: Dir = include_dir!("src/frames/idle");
static BLINKING_FRAMES_DIR: Dir = include_dir!("src/frames/blinking");

fn get_frames(dir: &'static Dir<'static>) -> Vec<&'static str> {
    dir.files()
        .filter_map(|f| f.contents_utf8())
        .collect()
}

#[derive(Debug)]
pub enum State {
    TALKING,
    IDLE,
    DYING,
    BLINKING,
}

pub struct Animation {
    last_talking_frame: Vec<String>,
    dying_frames: Vec<&'static str>,
    blinking_frames: Vec<&'static str>,
    idle_frames: Vec<&'static str>,
    last_time: i64,
    state: State,
    talking_frame_num: usize,
    change_talking_frame: bool,
    until_talking: i32,
    blink_frame_num: usize,
    rng: ThreadRng,
    timer: u32,
}

impl Animation {
    pub fn new() -> Self {
        Self {
            last_talking_frame: vec!["".to_string()],
            dying_frames: get_frames(&DYING_FRAMES_DIR),
            idle_frames: get_frames(&IDLE_FRAMES_DIR),
            blinking_frames: get_frames(&BLINKING_FRAMES_DIR),
            last_time: Utc::now().timestamp_millis(),
            state: State::IDLE,
            talking_frame_num: 0,
            change_talking_frame: false,
            until_talking: 0,
            blink_frame_num: 0,
            rng: rand::thread_rng(),
            timer: 1000,
        }
    }

    pub fn render_ascii_art_widget(&mut self, animation_area: Rect, frame: &mut Frame) {
        let box_width = animation_area.width as usize;
        let box_height = animation_area.height as usize;
        let border_block =
            Block::default()
                .borders(Borders::ALL)
                .title("STEMM GPT...").fg(Color::Rgb(0, 255, 0));

        frame.render_widget(border_block.clone(), animation_area);
        let inner_animation_area = border_block.inner(animation_area);


        let padded_frame = match self.state {
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
                    .map(|line| Line::from(Span::styled(line, Style::default().fg(Color::Rgb(0, 255, 0)))))
                    .collect();
                Text::from(text_lines)
            },
            State::IDLE => {
                if self.rng.gen_range(0..100) == 0  {
                    self.last_time = Utc::now().timestamp_millis();
                    self.talking_frame_num += 1;
                    self.set_state(State::BLINKING);
                }

                let current_frame = self.idle_frames[0];
                Text::from(Self::pad_ascii_frame(current_frame, box_width, box_height))
            },
            State::DYING => {
                if self.timer > 0 {
                    self.timer -= 1;
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
                        .map(|line| Line::from(Span::styled(line, Style::default().fg(Color::Rgb(0, 255, 0)))))
                        .collect();
                    Text::from(text_lines)
                } else {
                    let current_frame = self.dying_frames[0];
                    Text::from(Self::pad_ascii_frame(current_frame, box_width, box_height))
                }
            },
            State::BLINKING => {
                if Utc::now().timestamp_millis() - self.last_time > 50  {
                    self.last_time = Utc::now().timestamp_millis();
                    self.blink_frame_num += 1;
                }

                if self.blink_frame_num >= self.blinking_frames.len() {
                    self.blink_frame_num = 0;
                    self.state = State::IDLE;
                }

                let current_frame = self.blinking_frames[self.blink_frame_num];
                Text::from(Self::pad_ascii_frame(current_frame, box_width, box_height))
            },
        };

        frame.render_widget(Paragraph::new(padded_frame)
            .alignment(Alignment::Left),
            inner_animation_area)

    }

    fn pad_ascii_frame(frame: &str, target_width: usize, target_height: usize) -> String {
        // Split frame into lines and compute width and height
        let lines: Vec<&str> = frame.lines().collect();
        let frame_height = lines.len();
        let frame_width = lines.iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0);

        // Horizontal padding
        let total_padding = target_width.saturating_sub(frame_width);
        let left_pad = total_padding / 2;
        let right_pad = total_padding - left_pad;

        // Vertically center by adding blank lines above and below
        let total_vertical_padding = target_height.saturating_sub(frame_height);
        let top_pad = total_vertical_padding / 2;
        let bottom_pad = total_vertical_padding - top_pad;

        // Build the final padded frame
        let mut padded_lines = Vec::new();

        // Add top padding
        padded_lines.extend(std::iter::repeat(" ".repeat(target_width)).take(top_pad));

        // Pad each line horizontally
        for line in &lines {
            let padded_line = format!(
                "{}{}{}",
                " ".repeat(left_pad),
                line,
                " ".repeat(right_pad)
            );
            padded_lines.push(padded_line);
        }

        // Add bottom padding
        padded_lines.extend(std::iter::repeat(" ".repeat(target_width)).take(bottom_pad));

        // Join everything into the final string
        padded_lines.join("\n")
    }


    pub fn set_state(&mut self, state: State) {
        match self.state {
            State::DYING => {},
            State::BLINKING => {},
            _ => self.state = state,
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
            if self.until_talking == 0 {
                self.talking_frame_num = self.rng.gen_range(MIN/10000..MAX/10000);
            }

            return lines;
        }

        let center = height / 2;
        let mut amplitudes = Vec::new();

        // Control speech-like pauses and bursts
        let is_silent: bool = self.rng.gen_bool(0.2);
        let burst_strength: f32 = if is_silent {
            self.until_talking = 3;
            0.2
        } else {
            self.rng.gen_range(0.5..1.5)
        };

        for x in 0..width {
            let t = (x as f32 + frame as f32 * 0.5) * 0.3;

            let wave = (t.sin() * 0.5 + (t * 3.0).sin() * 0.3) * burst_strength;
            let jitter = self.rng.gen_range(-0.2..0.2);
            let base_amplitude = wave + jitter;

            // Push amplitude (clamped to avoid overflow)
            let amplitude = (base_amplitude * center as f32).clamp(-(center as f32), center as f32);
            amplitudes.push(amplitude as i32);
        }

        let avg_amplitude: f32 = amplitudes.iter().map(|&x| x as f32).sum::<f32>() / amplitudes.len() as f32;
        if avg_amplitude.abs() > 0.1 {
            Self::play_sound(avg_amplitude);
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

    fn play_sound(amplitude: f32) {
        std::thread::spawn(move || {
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Sink::try_new(&stream_handle).unwrap();

            let freq = 220.0 + amplitude * 300.0;
            let source = SineWave::new(freq)
                .take_duration(Duration::from_millis(200))
                .amplify(amplitude.clamp(0.1, 0.3));

            sink.append(source);
            sink.sleep_until_end();
        });
    }

}
