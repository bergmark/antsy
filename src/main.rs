#[macro_use]
extern crate derive_more;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    collections::{HashMap, VecDeque},
    error::Error,
    io,
    time::{Duration, Instant},
};
use strum::*;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect, *},
    style::*,
    text::*,
    widgets::{Block, Borders, *},
    Frame, Terminal,
};

mod float;
use float::Float;
mod bar;
use bar::Bar;
mod upgrade;
use upgrade::{GlobalUpgrade, Upgrade};

pub struct App {
    bars: VecDeque<Bar>,
    tick: Instant,
    last_bar_spawn: Option<Instant>,
    bars_to_spawn: usize,
    highlight: Option<Highlight>,
    last_bar_number: usize,
    global_upgrades: HashMap<GlobalUpgrade, usize>,
}

#[derive(Copy, Clone, Debug)]
enum Highlight {
    Bar { upgrade: Upgrade, row: usize },
    Global { upgrade: GlobalUpgrade },
}

struct UpgradeCost {
    target: usize,
    cost: Float,
}

impl App {
    fn new() -> App {
        App {
            bars: VecDeque::new(),
            tick: Instant::now(),
            last_bar_spawn: None,
            bars_to_spawn: 4,
            highlight: None,
            last_bar_number: 0,
            global_upgrades: GlobalUpgrade::iter().map(|g| (g, 0)).collect(),
        }
    }

    fn get_global_upgrade(&self, upgrade: GlobalUpgrade) -> Float {
        self.global_upgrades[&upgrade].into()
    }

    fn spawn_bar(&mut self) {
        self.last_bar_number += 1;
        self.bars.push_front(Bar::new(self.last_bar_number));
        if let Some(Highlight::Bar { row, upgrade: _ }) = &mut self.highlight {
            *row += 1;
        }
    }

    fn global_upgrade_price(&self, upgrade: GlobalUpgrade) -> Option<Float> {
        if self.bars.is_empty() {
            return None;
        }

        let level = self.global_upgrades[&upgrade];
        let cost = upgrade.cost(level);
        if self.bars[self.bars.len() - 1].gathered >= cost {
            Some(cost)
        } else {
            None
        }
    }

    fn can_afford_global(&self, upgrade: GlobalUpgrade) -> bool {
        self.global_upgrade_price(upgrade).is_some()
    }

    fn upgrade_price(&self, row: usize, upgrade: Upgrade) -> Option<UpgradeCost> {
        let cost = self.bars[row].upgrade_cost(upgrade);
        let target = row as i64 - upgrade.cost_target();
        if target >= 0 && self.bars[target as usize].gathered >= cost {
            Some(UpgradeCost {
                cost,
                target: target as usize,
            })
        } else {
            None
        }
    }

    fn can_afford(&self, row: usize, upgrade: Upgrade) -> bool {
        self.upgrade_price(row, upgrade).is_some()
    }

    fn purchase_upgrade(&mut self) {
        if let Some(Highlight::Bar { upgrade, row }) = self.highlight {
            if let Some(upgrade_cost) = self.upgrade_price(row, upgrade) {
                self.bars[row].inc_upgrade(upgrade);
                self.bars[upgrade_cost.target].gathered -= upgrade_cost.cost;
            }
        }
        if let Some(Highlight::Global { upgrade }) = self.highlight {
            if let Some(upgrade_cost) = self.global_upgrade_price(upgrade) {
                *self
                    .global_upgrades
                    .entry(upgrade)
                    .or_insert_with(|| panic!("Should have been init'd")) += 1;
                let last_i = self.bars.len() - 1;
                self.bars[last_i].gathered -= upgrade_cost;
                if let GlobalUpgrade::ProgressBars = upgrade {
                    self.bars_to_spawn += 2;
                }
            }
        }
    }

    fn on_tick(&mut self, now: Instant) {
        self.tick = now;

        if self.bars_to_spawn > 0
            && (self.last_bar_spawn.is_none()
                || now - self.last_bar_spawn.unwrap() >= Duration::from_secs(1))
        {
            self.spawn_bar();
            self.bars_to_spawn -= 1;
            self.last_bar_spawn = Some(self.tick);
        }

        for i in 0..self.bars.len() {
            let read_bar = self.bars.get(i).unwrap().clone();
            let done = self.bars[i].inc(
                self.global_upgrades[&GlobalUpgrade::Speed],
                self.global_upgrades[&GlobalUpgrade::ExpGain],
                self.global_upgrades[&GlobalUpgrade::ExpBoost],
                now,
            );
            if done {
                let gain = read_bar.gain(self);
                self.bars[i].gathered += gain;

                if i + 1 < self.bars.len() {
                    let transferred = self.bars[i].gathered * self.bars[i].transfer_ratio;
                    let gained = read_bar.gain(self) - transferred;
                    self.bars[i + 1].gathered += transferred;
                    self.bars[i].gathered -= transferred;
                    self.bars[i].last_completion = Some(Completion {
                        gain: gained,
                        transferred: Some(transferred),
                        tick: now,
                    });
                } else {
                    self.bars[i].last_completion = Some(Completion {
                        gain: read_bar.gain(self),
                        transferred: None,
                        tick: now,
                    });
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
pub struct Completion {
    gain: Float,
    transferred: Option<Float>,
    tick: Instant,
}

fn exp_for_level(level: usize) -> usize {
    1.5f64.powf((level - 1) as f64) as usize
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    log("\nStarting");
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(40);
    let app = App::new();
    let res = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
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
            terminal.draw(|f| ui(f, &app))?;
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

fn mk_gauge(bar: &Bar, color: Color) -> Gauge<'static> {
    Gauge::default()
        // .block(Block::default().borders(Borders::ALL).title("Progress"))
        .gauge_style(
            Style::default().fg(color).bg(Color::Black), // .add_modifier(Modifier::ITALIC),
        )
        .percent(bar.progress.0 as u16)
}

fn mk_text_line(text: &str) -> Paragraph<'_> {
    Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .wrap(Wrap { trim: true })
}

fn bar_values<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) {
    let chunk = render_border(f, chunk, "Values");
    let chunks = rect_to_lines(chunk);
    for (i, chunk) in chunks.into_iter().enumerate() {
        if i < app.bars.len() {
            let bar = &app.bars[i];
            f.render_widget(mk_text_line(&format!("{}", bar.gathered)), chunk);
        }
    }
}

fn mk_button(label: &str, highlight: bool, can_afford: bool) -> Paragraph<'static> {
    let color = if highlight {
        Color::Yellow
    } else {
        Color::White
    };
    let modifier = if can_afford {
        Modifier::BOLD | Modifier::UNDERLINED
    } else {
        Modifier::empty()
    };
    let text = Span::styled(
        format!("| {label} |"),
        Style::default().fg(color).add_modifier(modifier),
    );
    Paragraph::new(text)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true })
}

fn rect_to_lines(r: Rect) -> Vec<Rect> {
    (0..r.height)
        .map(|i| Rect {
            x: r.x,
            y: r.y + i,
            height: 1,
            width: r.width,
        })
        .collect()
}

fn log(s: &str) {
    use std::fs::*;
    use std::io::*;
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("log.txt")
        .unwrap();

    if let Err(e) = writeln!(file, "{}", s) {
        eprintln!("Couldn't write to file: {}", e);
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Length(10)])
        .split(f.size());

    let top = chunks[0];
    let bottom = chunks[1];

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length(20),     // bars
                Constraint::Length(10),     // values
                Constraint::Length(20),     // transferred
                Constraint::Length(12),     // level
                Constraint::Length(12),     // speed
                Constraint::Percentage(50), // upgrades
            ]
            .as_ref(),
        )
        .split(top);

    let bars = chunks[0];
    let values = chunks[1];
    let transferred = chunks[2];
    let level = chunks[3];
    let speed = chunks[4];
    let bar_upgrades = chunks[5];

    render_bars(f, app, bars);
    bar_values(f, app, values);
    render_transferred(f, app, transferred);
    render_level(f, app, level);
    render_speed(f, app, speed);
    render_bar_upgrades(f, app, bar_upgrades);
    render_global_upgrades(f, app, bottom);
}

fn render_border<B: Backend>(f: &mut Frame<B>, chunk: Rect, title: &str) -> Rect {
    f.render_widget(
        Block::default()
            .title(title)
            .style(Style::default().bg(Color::Black))
            .borders(Borders::ALL),
        chunk,
    );
    chunk.inner(&Margin {
        horizontal: 1,
        vertical: 1,
    })
}

fn render_text<B: Backend>(f: &mut Frame<B>, chunk: Rect, text: &str) {
    let w = Paragraph::new(text)
        .style(Style::default().fg(Color::White).bg(Color::Black))
        .wrap(Wrap { trim: false });

    f.render_widget(w, chunk);
}

fn render_transferred<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) {
    let chunk = render_border(f, chunk, "Transfer");
    let chunks = rect_to_lines(chunk);
    for (bar, chunk) in app.bars.iter().zip(chunks.into_iter()) {
        if let Some(completion) = bar.recent_completion(app.tick) {
            let gain = completion.gain;
            match completion.transferred {
                None => render_text(f, chunk, &format!("+{gain:.3}")),
                Some(transferred) => {
                    let sig = if gain < 0. { "" } else { "+" };
                    render_text(f, chunk, &format!("{sig}{gain:.3} / ↓{transferred:.3}"))
                }
            }
        }
    }
}

fn render_bars<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) {
    let chunk = render_border(f, chunk, "Bars");
    let colors = vec![Color::Blue, Color::White, Color::Green, Color::Red];
    let chunks: Vec<_> = rect_to_lines(chunk);
    for (i, (bar, chunk)) in app.bars.iter().zip(chunks.into_iter()).enumerate() {
        let color = if bar.is_boosted(app.tick) {
            Color::Yellow
        } else {
            colors[i % colors.len()]
        };
        f.render_widget(mk_gauge(bar, color), chunk);
    }
}

impl Upgrade {
    fn label(self, number: usize, level: usize) -> String {
        use Upgrade::*;
        let cost = self.cost(level);
        match self {
            Speed => format!("x1.3 SPD: {cost}"),
            Gain => format!("+1: {cost}"),
            Double => format!("x2: {cost} from #{}", number + 1),
            Triple => format!("x3: {cost} from #{}", number + 4),
            Quadruple => format!("x4: {cost} from #{}", number + 7),
        }
    }
}

impl GlobalUpgrade {
    fn label(self, level: usize) -> String {
        use GlobalUpgrade::*;
        let cost = self.cost(level);
        match self {
            Speed => format!("+5% SPD | {cost} "),
            ExpBoost => format!("+1s Level Up Boost | {cost}"),
            ProgressBars => format!("2 Progress Bars | {cost}"),
            Gain => format!("+1 Gain | {cost}"),
            ExpGain => format!("+1 Exp Gain | {cost}"),
        }
    }
}

fn render_speed<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) {
    let chunk = render_border(f, chunk, "Speed");
    let chunks = rect_to_lines(chunk);
    for (_i, (bar, chunk)) in app.bars.iter().zip(chunks.into_iter()).enumerate() {
        let speed = bar.speed(app.global_upgrades[&GlobalUpgrade::Speed]);
        let num = format_num::NumberFormat::new();
        render_text(f, chunk, &num.format(".3%", speed));
    }
}

fn render_level<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) {
    let chunk = render_border(f, chunk, "Level");
    let chunks = rect_to_lines(chunk);
    for (_i, (bar, chunk)) in app.bars.iter().zip(chunks.into_iter()).enumerate() {
        let level = bar.level;
        let exp = bar.exp;
        let to_level = bar.exp_for_next_level();
        render_text(f, chunk, &format!("L{level} {exp}/{to_level}"));
    }
}

fn render_bar_upgrades<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) {
    let chunk = render_border(f, chunk, "Upgrades");
    let chunks = rect_to_lines(chunk);
    for (i, (bar, chunk)) in app.bars.iter().zip(chunks.into_iter()).enumerate() {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                (0..Upgrade::COUNT)
                    .map(|_| Constraint::Ratio(1, Upgrade::COUNT as u32))
                    .collect::<Vec<_>>(),
            )
            .split(chunk);
        for (upgrade, chunk) in Upgrade::iter().zip(chunks.into_iter()) {
            let highlight = match app.highlight {
                None | Some(Highlight::Global { .. }) => false,
                Some(Highlight::Bar {
                    row: highlight_row,
                    upgrade: highlight_upgrade,
                }) => i == highlight_row && upgrade == highlight_upgrade,
            };
            let can_afford = app.can_afford(i, upgrade);
            let button = mk_button(
                &upgrade.label(bar.number, bar.get_upgrade_u(upgrade)),
                highlight,
                can_afford,
            );
            f.render_widget(button, chunk);
        }
    }
}

fn render_global_upgrades<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) {
    let chunk = render_border(f, chunk, "Global upgrades");
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            (0..GlobalUpgrade::COUNT)
                .map(|_| Constraint::Ratio(1, GlobalUpgrade::COUNT as u32))
                .collect::<Vec<_>>(),
        )
        .split(chunk);

    for (upgrade, chunk) in GlobalUpgrade::iter().zip(chunks.into_iter()) {
        let highlight = match app.highlight {
            None | Some(Highlight::Bar { .. }) => false,
            Some(Highlight::Global {
                upgrade: highlight_upgrade,
            }) => upgrade == highlight_upgrade,
        };
        let can_afford = app.can_afford_global(upgrade);
        let button = mk_button(
            &upgrade.label(app.global_upgrades[&upgrade]),
            highlight,
            can_afford,
        );
        f.render_widget(button, chunk);
    }
}
