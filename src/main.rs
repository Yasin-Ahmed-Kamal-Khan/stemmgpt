mod app;
use app::App;

use color_eyre::eyre::Report;
use ratatui::{prelude::CrosstermBackend, Terminal};

fn main() -> Result<(), Report> {
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