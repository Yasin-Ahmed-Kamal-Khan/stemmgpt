use std::panic;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{poll, read, Event, KeyCode},
    execute,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen
    },
};
use include_dir::{include_dir, Dir};
use std::{
    io::{self, stdout, Write},
    thread,
    time::Duration,
};

// Include the entire frames directory at compile time
static FRAMES_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/src/frames");

fn get_frames() -> Vec<String> {
    let mut frames = Vec::new();

    // Get all .txt files from the included directory
    let mut files = FRAMES_DIR
        .files()
        .filter(|file| {
            file.path().extension().and_then(|ext| ext.to_str()) == Some("txt")
        })
        .collect::<Vec<_>>();

    // Sort files by name to ensure proper sequence
    files.sort_by_key(|file| file.path().to_owned());

    // Load the contents of each file
    for file in files {
        if let Some(content) = file.contents_utf8() {
            frames.push(content.to_string());
        }
    }

    // Fallback to default frames if no files were found
    if frames.is_empty() {
        frames.push(
            r#"
    No animation
    frames found!

    Place .txt files
    in the frames
    directory.
            "#.to_string(),
        );
    }

    return frames
}

fn main() -> io::Result<()> {
    // Set up panic hook to clean up the terminal
    enable_raw_mode()?;

    panic::set_hook(Box::new(|panic_info| {
        disable_raw_mode().expect("Failed to disable raw mode on panic");
        eprintln!("Panic occurred: {}", panic_info);
    }));

    // Enter alternate screen and hide cursor
    execute!(stdout(), EnterAlternateScreen, Hide)?;

    // Load frames from the included directory
    let frames = get_frames();

    // Get terminal size
    let (width, height) = crossterm::terminal::size()?;

    let mut frame_index = 0;
    let mut running = true;

    while running {
        if poll(Duration::from_millis(100))? {
            match read()? {
                Event::Key(key_event) => {
                    if matches!(key_event.code, KeyCode::Char('q') | KeyCode::Char('Q')) {
                        running = false;
                    }
                }
                _ => {}
            }
        }        // Clear screen
        execute!(stdout(), Clear(ClearType::All))?;

        let _ = draw_to_terminal(&frames, frame_index, width, height);

        // Add instructions at the bottom
        execute!(
            stdout(),
            MoveTo(2, height - 2),
            crossterm::style::Print("Press 'q' to quit")
        )?;

        // Show frame count
        let frame_info = format!("Frame: {}/{}", frame_index + 1, frames.len());
        execute!(
            stdout(),
            MoveTo(width - frame_info.len() as u16 - 2, height - 2),
            crossterm::style::Print(frame_info)
        )?;

        // Flush to ensure drawing happens
        stdout().flush()?;

        // Wait a bit before showing the next frame
        thread::sleep(Duration::from_millis(200));

        // Move to next frame
        frame_index = (frame_index + 1) % frames.len();
    }

    // Clean up terminal
    execute!(stdout(), Show, LeaveAlternateScreen)?;

    let _ = disable_raw_mode();
    Ok(())
}


fn draw_to_terminal(frames: &Vec<String>, frame_index: usize, width: u16, height: u16) -> io::Result<()> {
    let width = width as usize;
    let height = height as usize;
    let frame = &frames[frame_index];
    let frame_lines: Vec<&str> = frame.lines().collect();
    let max_line_width = frame_lines.iter().map(|line| line.len()).max().unwrap_or(0);
    for (i, line) in frame_lines.iter().enumerate() {
        let x = (width - max_line_width) / 2;
        let y = (height - frame_lines.len()) / 2 + i;

        if y < height {
            execute!(
                stdout(),
                MoveTo(x as u16, y as u16),
                crossterm::style::Print(line)
            )?;
        }
    }

    Ok(())
}