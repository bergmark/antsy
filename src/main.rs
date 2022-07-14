#[macro_use]
extern crate derive_more;

use crossterm::{
    self,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

mod app;
mod bar;
mod float;
mod render;
mod save;
mod upgrade;

use app::{App, Highlight};
use bar::Bar;
use float::Float;
use upgrade::{GlobalUpgrade, Upgrade};

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    terminal::enable_raw_mode()?;
    log("\nStarting");
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(40);
    let app = App::load(Instant::now());
    let res = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    terminal::disable_raw_mode()?;
    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

#[derive(Copy, Clone, Debug)]
enum Dir {
    Down,
    Left,
    Right,
    Up,
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => {
                        app.save();
                        return Ok(());
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => app.purchase_upgrade(),
                    KeyCode::Tab => change_highlight_pane(&mut app),
                    KeyCode::Down => move_highlight(&mut app, Dir::Down),
                    KeyCode::Up => move_highlight(&mut app, Dir::Up),
                    KeyCode::Right => move_highlight(&mut app, Dir::Right),
                    KeyCode::Left => move_highlight(&mut app, Dir::Left),
                    _ => {}
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
            app.on_tick(last_tick);
            terminal.draw(|f| render::ui(f, &app))?;
        }
    }
}

fn change_highlight_pane(app: &mut App) {
    match app.highlight {
        None => move_highlight(app, Dir::Down),
        Some(Highlight::Global { .. }) => {
            app.highlight = Some(Highlight::Bar {
                upgrade: Upgrade::Speed,
                row: 0,
            })
        }
        Some(Highlight::Bar { .. }) => {
            app.highlight = Some(Highlight::Global {
                upgrade: GlobalUpgrade::Speed,
            })
        }
    }
}

fn move_highlight(app: &mut App, dir: Dir) {
    if app.bars.is_empty() {
        return;
    }

    app.highlight = Some(match app.highlight {
        None => Highlight::Bar {
            upgrade: Upgrade::Speed,
            row: 0,
        },
        Some(Highlight::Bar { upgrade, mut row }) => match dir {
            Dir::Left => Highlight::Bar {
                row,
                upgrade: upgrade.prev(),
            },
            Dir::Right => Highlight::Bar {
                row,
                upgrade: upgrade.next(),
            },
            Dir::Up => {
                if row > 0 {
                    row -= 1;
                } else {
                    row = app.bars.len() - 1;
                }
                Highlight::Bar { row, upgrade }
            }
            Dir::Down => {
                if row == app.bars.len() - 1 {
                    row = 0;
                } else {
                    row += 1;
                }
                Highlight::Bar { row, upgrade }
            }
        },
        Some(Highlight::Global { upgrade }) => match dir {
            Dir::Left => Highlight::Global {
                upgrade: upgrade.prev(),
            },
            Dir::Right => Highlight::Global {
                upgrade: upgrade.next(),
            },
            Dir::Down | Dir::Up => Highlight::Global { upgrade },
        },
    });
}

fn log(s: &str) {
    use std::fs::*;
    use std::io::*;

    if std::path::Path::new("log.txt").exists() {
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open("log.txt")
            .unwrap();

        if let Err(e) = writeln!(file, "{}", s) {
            eprintln!("Couldn't write to file: {}", e);
        }
    }
}
