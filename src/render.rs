use crate::app::{App, Highlight};
use crate::bar::Bar;
use crate::float::Float;
use crate::upgrade::{GlobalUpgrade, Upgrade};
use format_num::format_num;
use strum::*;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect, *},
    style::*,
    text::*,
    widgets::{Block, Borders, *},
    Frame,
};

const UPGRADE_0_WIDTH: u16 = "| x1.3 SPD: DD.DM |".len() as u16;
const UPGRADE_1_WIDTH: u16 = "| +1: DD.DM |".len() as u16;
const UPGRADE_2_WIDTH: u16 = "| x2: DD.DM from #NNN |".len() as u16;
const UPGRADE_3_WIDTH: u16 = "| x3: DD.DM from #NNN |".len() as u16;
const UPGRADE_4_WIDTH: u16 = "| x4: DD.DM from #NNN |".len() as u16;

pub(crate) fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(80), Constraint::Length(10)])
        .split(f.size());

    let top = chunks[0];
    let bottom = chunks[1];

    const BORDERS: u16 = 2;
    const VALUES_WIDTH: u16 = " DD.DM ".len() as u16;
    const TRANSFERRED_WIDTH: u16 = " +DD.DM / vDD.DM ".len() as u16;
    const LEVEL_WIDTH: u16 = 18;
    const SPEED_WIDTH: u16 = 12;
    const UPGRADES_WIDTH: u16 =
        UPGRADE_0_WIDTH + UPGRADE_1_WIDTH + UPGRADE_2_WIDTH + UPGRADE_3_WIDTH + UPGRADE_4_WIDTH;
    let bar_width: u16 = top.width
        - VALUES_WIDTH
        - TRANSFERRED_WIDTH
        - LEVEL_WIDTH
        - SPEED_WIDTH
        - UPGRADES_WIDTH
        - BORDERS * 5;

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Length(bar_width),
                Constraint::Length(VALUES_WIDTH + BORDERS),
                Constraint::Length(TRANSFERRED_WIDTH + BORDERS),
                Constraint::Length(LEVEL_WIDTH + BORDERS),
                Constraint::Length(SPEED_WIDTH + BORDERS),
                Constraint::Length(UPGRADES_WIDTH + BORDERS),
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
    assert_eq!(bar_upgrades.width, 103);

    render_bars(f, app, bars);

    render_bar_values(f, app, values);
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
        .alignment(Alignment::Center)
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
                None => render_text(f, chunk, &format_num!("+.3s", gain)),
                Some(transferred) => render_text(
                    f,
                    chunk,
                    &format!(
                        "{gain} / ↓{transferred}",
                        gain = format_num!("+.3s", gain),
                        transferred = format_num!(".3s", transferred)
                    ),
                ),
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
        let speed = bar.speed_multiplier(app.global_upgrades[&GlobalUpgrade::Speed]);
        let speed = ((speed.0 * 100.) as usize as f64) / 100.;
        let num = format_num::NumberFormat::new();
        render_text(f, chunk, &format!("x {}", &num.format(".2", speed)));
    }
}

fn render_level<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) {
    let chunk = render_border(f, chunk, "Level");
    let chunks = rect_to_lines(chunk);
    for (_i, (bar, chunk)) in app.bars.iter().zip(chunks.into_iter()).enumerate() {
        let level = bar.level;
        let exp = Float::from(bar.exp);
        let to_level = Float::from(bar.exp_for_next_level());
        render_text(f, chunk, &format!("L{level} {exp}/{to_level}"));
    }
}

fn render_bar_upgrades<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) {
    let chunk = render_border(f, chunk, "Upgrades");
    let chunks = rect_to_lines(chunk);
    assert_eq!(Upgrade::COUNT, 5);
    for (i, (bar, chunk)) in app.bars.iter().zip(chunks.into_iter()).enumerate() {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                (0..Upgrade::COUNT)
                    .map(|i| {
                        Constraint::Length(match i {
                            0 => UPGRADE_0_WIDTH,
                            1 => UPGRADE_1_WIDTH,
                            2 => UPGRADE_2_WIDTH,
                            3 => UPGRADE_3_WIDTH,
                            4 => UPGRADE_4_WIDTH,
                            _ => unreachable!(),
                        })
                    })
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

fn mk_gauge(bar: &Bar, color: Color) -> Gauge<'static> {
    Gauge::default()
        // .block(Block::default().borders(Borders::ALL).title("Progress"))
        .gauge_style(
            Style::default().fg(color).bg(Color::Black), // .add_modifier(Modifier::ITALIC),
        )
        .percent(bar.progress.0 as u16)
}

fn mk_text_line_fg(fg_color: Color, text: &str) -> Paragraph<'_> {
    Paragraph::new(text)
        .alignment(Alignment::Center)
        .style(Style::default().fg(fg_color).bg(Color::Black))
        .wrap(Wrap { trim: true })
}

fn render_bar_values<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) {
    let chunk = render_border(f, chunk, "Values");
    let chunks = rect_to_lines(chunk);
    let highlight_cost_target = app.highlight_cost_target();
    for (i, chunk) in chunks.into_iter().enumerate() {
        if i >= app.bars.len() {
            break;
        }

        let color = if highlight_cost_target.map_or(false, |t| t == i as i64) {
            Color::Yellow
        } else {
            Color::White
        };

        let bar = &app.bars[i];

        f.render_widget(mk_text_line_fg(color, &format!("{}", bar.gathered)), chunk);
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
        format!("  {label}  "),
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
