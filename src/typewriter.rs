use chrono::Utc;
use ratatui::{layout::Alignment, style::{Color, Style}, widgets::{Block, Paragraph, Widget, Wrap}};

use crate::animation::State;

pub struct Typewriter {
    current_message_index: usize,
    visible_chars: usize,
    last_char_time: i64,
    char_delay_ms: i64,
    messages: Vec<String>,
}

impl Typewriter {
    pub fn new() -> Self {
        Self {
            current_message_index: 0,
            visible_chars: 0,
            last_char_time: Utc::now().timestamp_millis(),
            char_delay_ms: 75,
            messages: Vec::new(), // 50ms between chars (adjust for speed)
        }
    }

    // Call this method in your main loop to update the typewriter effect
    pub fn update_typewriter(&mut self) -> Option<State> {
        if self.messages.is_empty() {
            return Some(State::IDLE)
        }

        let current_time = Utc::now().timestamp_millis();

        // Check if enough time has passed to show next character
        if current_time - self.last_char_time >= self.char_delay_ms {
            let current_message = &self.messages[self.current_message_index];
            //println!("Typewriter: visible_chars={}, total_chars={}",
              //  self.visible_chars,
                //current_message.chars().count();

            if self.visible_chars < current_message.chars().count() {
                // Show next character
                self.visible_chars += 1;
                self.last_char_time = current_time;
                return Some(State::TALKING)
            } else {
                return Some(State::IDLE)
            }
        }

        None
    }

    fn start_new_message(&mut self) {
        if !self.messages.is_empty() {
            self.current_message_index = self.messages.len() - 1;
            self.visible_chars = 0;
            self.last_char_time = Utc::now().timestamp_millis();
        }
    }

    pub fn output_canvas(&mut self) -> impl Widget + '_ {
        let block = Block::bordered()
            .title(" Output ")
            .title_alignment(Alignment::Left)
            .style(Style::default().fg(Color::Rgb(0, 0, 255)).bg(Color::White));

        let display_text = if self.messages.is_empty() {
            String::new()
        } else {
            let current_message = &self.messages[self.current_message_index];
            current_message.chars().take(self.visible_chars).collect::<String>()
        };

        Paragraph::new(display_text)
            .block(block)
            .wrap(Wrap { trim: false })
    }

    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);
        self.start_new_message();
    }
}