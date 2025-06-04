mod app;
mod typewriter;
mod animation;

use app::App;
use color_eyre::eyre::Report;
use ratatui::{prelude::CrosstermBackend, Terminal};
use std::fs;
use std::time::Duration;
use std::thread;

fn main() -> Result<(), Report> {
    color_eyre::install()?;

    // Wait for Python to be ready
    println!("Waiting for AI model to load...");
    while fs::metadata("ready.txt").is_err() {
        thread::sleep(Duration::from_millis(100));
    }
    println!("AI model ready!");

    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let result = App::new().run(terminal);

    let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen);
    let _ = crossterm::terminal::disable_raw_mode();

    result
}