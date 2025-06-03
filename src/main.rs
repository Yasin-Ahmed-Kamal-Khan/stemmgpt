mod app;
mod typewriter;
mod animation;
use std::process::exit;

use app::App;

use pyo3::prelude::*;
use pyo3::types::PyModule;

use color_eyre::eyre::Report;
use ratatui::{prelude::CrosstermBackend, Terminal};

use std::env;

fn main() -> Result<(), Report> {
    color_eyre::install()?;

    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(std::io::stdout(), crossterm::terminal::EnterAlternateScreen)?;

    env::set_var("PYTHONPATH", "C:\\dev\\rust\\stemmgpt\\venv\\Lib\\site-packages");
    env::set_var("VIRTUAL_ENV", "C:\\dev\\rust\\stemmgpt\\venv");

    /*env::set_var("PYTHONPATH", r".\venv\Lib\site-packages");
    Python::with_gil(|py| {
        println!("Python executable: {:?}", py.run("import sys; print(sys.executable)", None, None));
        println!("Python path: {:?}", py.run("import sys; print(sys.prefix)", None, None));
    });
    exit(0);*/

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

        
        // 2. Call the function with input (using self.input as the message)
        // 3. Convert Python result to Rust Strin
        App::new(ai_module.into()).run(terminal)
    });

    //let result = App::new().run(terminal);

    let _ = crossterm::execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen);
    let _ = crossterm::terminal::disable_raw_mode();

    result
}