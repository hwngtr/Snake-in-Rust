mod game;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    style::{self, Color, Print, Stylize},
    terminal::{self},
    ExecutableCommand,
};
use game::{Cell, Direction, Game};
use std::env;
use std::io::{self, stdout, Write};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("usage: snake <GROWS: 0|1> [BOARD STRING]");
        return Ok(());
    }

    let grow_on_eat = match args[1].as_str() {
        "1" => true,
        "0" => false,
        _ => {
            println!("snake_grows must be either 1 (grows) or 0 (does not grow)");
            return Ok(());
        }
    };

    let mut game = if args.len() >= 3 {
        match Game::from_string(&args[2], grow_on_eat) {
            Ok(g) => g,
            Err(e) => {
                println!("Error parsing board: {}", e);
                return Ok(());
            }
        }
    } else {
        Game::new(20, 10, grow_on_eat)
    };

    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;

    render(&mut stdout, &game)?;

    let mut last_dir = Direction::Right;

    while !game.game_over {
        let timeout = Duration::from_millis(300);
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => break,
                        KeyCode::Up => last_dir = Direction::Up,
                        KeyCode::Down => last_dir = Direction::Down,
                        KeyCode::Left => last_dir = Direction::Left,
                        KeyCode::Right => last_dir = Direction::Right,
                        _ => {}
                    }
                }
            }
        }

        game.update(last_dir);
        render(&mut stdout, &game)?;
    }

    execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show)?;
    terminal::disable_raw_mode()?;

    println!("Game Over! Score: {}", game.score);

    Ok(())
}

fn render(stdout: &mut io::Stdout, game: &Game) -> io::Result<()> {
    stdout.execute(cursor::MoveTo(0, 0))?;
    stdout.execute(Print(format!("SCORE: {}   \r\n", game.score)))?; // Header

    for y in 0..game.height {
        for x in 0..game.width {
            let idx = y * game.width + x;
            let cell = game.cells[idx];

            match cell {
                Cell::Snake => {
                    stdout.execute(style::PrintStyledContent("S".with(Color::Yellow)))?;
                }
                Cell::Food => {
                    stdout.execute(style::PrintStyledContent("O".with(Color::Red)))?;
                }
                Cell::Wall => {
                    stdout.execute(style::PrintStyledContent("\u{2588}".with(Color::Blue)))?;
                }
                Cell::Plain => {
                    stdout.execute(Print(" "))?;
                }
            }
        }
        stdout.execute(Print("\r\n"))?;
    }
    stdout.flush()?;
    Ok(())
}
