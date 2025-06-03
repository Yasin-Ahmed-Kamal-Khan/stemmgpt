mod app;
mod typewriter;
mod animation;
use app::App;

use pyo3::prelude::*;
use pyo3::types::PyModule;

use color_eyre::eyre::Report;
use ratatui::{prelude::CrosstermBackend, Terminal};

fn main() -> Result<(), Report> {
    color_eyre::install()?;

    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let result = Python::with_gil(|py| {
        // Add current directory to sys.path
        let sys = py.import("sys")?;
        let path = std::env::current_dir()?; // current working directory
        sys.getattr("path")?
            .call_method1("insert", (0, path.to_str().unwrap()))?;

        let ai_module = PyModule::import(py, "ai")?;
        App::new(ai_module.into()).run(terminal)
    });

    //let result = App::new().run(terminal);

    let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen);
    let _ = crossterm::terminal::disable_raw_mode();

    result
}