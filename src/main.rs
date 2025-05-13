use std::{thread, time::Duration};
use clap::{Parser, ValueEnum};
use crossterm::{
    cursor, queue,
    style::{Color, Print, SetForegroundColor},
    terminal::{Clear, ClearType},
    ExecutableCommand,
};
use rand::Rng;
use std::io::{stdout, Write};

#[derive(Parser)]
#[command(version, about = "An animated cowsay-like program")]
struct Cli {
    /// The message for the cow to say
    message: String,

    /// Animation style
    #[arg(short, long, value_enum, default_value_t = AnimationStyle::Blink)]
    animation: AnimationStyle,

    /// Animation speed in milliseconds
    #[arg(short = 's', long, default_value_t = 300)]
    speed: u64,
}

#[derive(Copy, Clone, ValueEnum, Debug)]
enum AnimationStyle {
    Blink,
    Wave,
    ColorCycle,
    Random,
}

fn main() -> std::io::Result<()> {
    let args = Cli::parse();
    let mut stdout = stdout();

    // Hide cursor during animation
    stdout.execute(cursor::Hide)?;

    // Animation loop
    for i in 0..20 {
        // Clear previous frame
        queue!(
            stdout,
            Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )?;

        // Draw speech bubble
        draw_speech_bubble(&args.message, &mut stdout, i, &args)?;

        // Draw animated cow
        draw_cow(&mut stdout, i, &args)?;

        stdout.flush()?;
        thread::sleep(Duration::from_millis(args.speed));
    }

    // Show cursor again
    stdout.execute(cursor::Show)?;

    Ok(())
}

fn draw_speech_bubble(
    message: &str,
    stdout: &mut std::io::Stdout,
    frame: i32,
    args: &Cli,
) -> std::io::Result<()> {
    let lines: Vec<&str> = message.split('\n').collect();
    let max_length = lines.iter().map(|l| l.len()).max().unwrap_or(0);

    // Top border
    queue!(stdout, Print(" "))?;
    for _ in 0..max_length + 2 {
        match args.animation {
            AnimationStyle::ColorCycle => {
                let color = Color::Rgb {
                    r: ((frame * 10) % 255) as u8,
                    g: ((frame * 20) % 255) as u8,
                    b: ((frame * 30) % 255) as u8,
                };
                queue!(stdout, SetForegroundColor(color), Print("═"),)?;
            }
            _ => queue!(stdout, Print("═"))?,
        }
    }
    queue!(stdout, Print("\n"))?;

    // Message lines
    for line in &lines {
        queue!(stdout, Print(" "))?;
        match args.animation {
            AnimationStyle::Wave => {
                for (i, c) in line.chars().enumerate() {
                    let y_offset = ((frame as f32 * 0.5 + i as f32 * 0.3).sin() * 2.0) as i32;
                    queue!(
                        stdout,
                        cursor::MoveTo(2 + i as u16, 1 + y_offset as u16),
                        Print(c)
                    )?;
                }
            }
            _ => queue!(stdout, Print(format!(" {} ", line)))?,
        }
        queue!(stdout, Print("\n"))?;
    }

    // Bottom border
    queue!(stdout, Print(" "))?;
    for _ in 0..max_length + 2 {
        queue!(stdout, Print("═"))?;
    }
    queue!(stdout, Print("\n"))?;

    Ok(())
}

fn draw_cow(stdout: &mut std::io::Stdout, frame: i32, args: &Cli) -> std::io::Result<()> {
    // println!("args.animation = {:?}", args.animation);

    let cow_art = match args.animation {
        AnimationStyle::Blink if frame % 4 < 2 => r"
     \   ^__^
      \  (oo)\_______
         (__)\       )\/\
             ||----w |
             ||     ||
        ".to_string(),
        AnimationStyle::Random => {
            let mut rng = rand::thread_rng();
            let eyes = if rng.gen_bool(0.5) { "oo" } else { "^^" };
            format!(

r"              ^__^
                ({})\_______
                (__)\       )\/\
                    ||----w |
                    ||     ||
        ",
                eyes
            )
        } // trigger this case with cargo run -- --animation random
        _ => r"
     \   ^__^
      \  (oo)\_______
         (__)\       )\/\
             ||----w |
             ||     ||
        ".to_string(),
    };

    for (i, line) in cow_art.trim_start().lines().enumerate() {
        queue!(
            stdout,
            cursor::MoveTo(0, 3 + i as u16),
            Print(line)
        )?;
    }

    Ok(())
}