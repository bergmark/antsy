#[macro_use]
extern crate derive_more;

use crossterm::{
    self,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use structopt::StructOpt;
use tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};

mod app;
mod bar;
mod controls;
mod float;
mod opts;
mod render;
mod save;
mod upgrade;

use app::App;
use bar::Bar;
use controls::Action;
use float::Float;
use opts::Opts;
use upgrade::{GlobalUpgrade, Upgrade};

fn main() -> Result<(), Box<dyn Error>> {
    let opts = Opts::from_args();
    let app = App::load(opts, Instant::now());

    // setup terminal
    terminal::enable_raw_mode()?;
    log("\nStarting");
    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(40);
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
                match app.ui_state.handle_keypress(key, app.bars.len()) {
                    Action::PurchaseUpgrade => app.purchase_upgrade(),
                    Action::Quit => {
                        app.save();
                        return Ok(());
                    }
                    Action::Noop => (),
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
