use strum::*;
use tui::{
    backend::Backend,
    layout::*,
    // style::*,
    // text::*,
    // widgets::*,
    Frame,
};

use crate::app::App;
use crate::prestige::PrestigeUpgrade;
use crate::render::util::*;
use crate::ui::prestige::Highlight;
use crate::ui::Prestige;

pub(crate) fn render<B: Backend>(f: &mut Frame<B>, app: &App, ui_state: Prestige) {
    let chunks = render_border(f, f.size(), "Prestige");
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks);

    let left = chunks[0];
    let right = chunks[1];

    render_prestige_stats(f, app, ui_state, left);
    render_prestige_upgrades(f, app, ui_state, right);
}

fn render_prestige_stats<B: Backend>(
    f: &mut Frame<B>,
    app: &App,
    ui_state: Prestige,
    chunks: Rect,
) {
    let chunks = rect_to_lines(chunks);
    let can_prestige = app.prestige.can_prestige(app.bars.len());
    if can_prestige {
        render_left_text(
            f,
            chunks[0],
            &format!("Current prestige points: {}", app.prestige.current),
        );
        render_left_text(
            f,
            chunks[1],
            &format!(
                "Points to claim on prestige: {}",
                app.prestige.claimable_prestige(app.bars.len())
            ),
        );
    } else {
        render_text(f, chunks[0], "You cannot prestige until you reach 10 bars");
    }

    // chunks[2]

    f.render_widget(
        mk_button(
            "Prestige",
            Highlight::PrestigeButton == ui_state.highlight,
            app.prestige.can_prestige(app.bars.len()),
        ),
        chunks[3],
    );

    // chunks[4]

    render_text(
        f,
        chunks[5],
        &format!(
            "Current prestige points: {points}",
            points = app.prestige.current
        ),
    );
}

fn render_prestige_upgrades<B: Backend>(
    f: &mut Frame<B>,
    app: &App,
    ui_state: Prestige,
    chunks: Rect,
) {
    let chunks = rect_to_lines(chunks);
    for (i, (upgrade, chunk)) in PrestigeUpgrade::iter().zip(chunks.into_iter()).enumerate() {
        let cost = if app.prestige.is_max_level(upgrade) {
            "MAXED".to_owned()
        } else {
            app.prestige.cost(upgrade).to_string()
        };
        let text = format!("{label}: {cost}", label = prestige_upgrade_label(upgrade),);
        let button = mk_button_align(
            &text,
            Highlight::Upgrade(i) == ui_state.highlight,
            app.prestige.can_afford(upgrade),
            Alignment::Left,
        );
        f.render_widget(button, chunk)
    }
}

fn prestige_upgrade_label(upgrade: PrestigeUpgrade) -> &'static str {
    use PrestigeUpgrade::*;
    match upgrade {
        CompleteFaster => "Complete bars 5% sooner",
        LevelUpFaster => "Level up 5% quicker",
        TransferExtraExp => "Transfer 1% of exp if overleveled",
        TransferExtraValue => "Transfer 1% of value if overvalued",
        UpgradeAnyButton => "Upgrade button upgrades more",
        AutomateGlobalSpeed => "Automate global speed upgrading",
        AutomateGlobalExpBoost => "Automate exp boost",
        AutomateProgressBars => "Automate progress bar purhases",
        AutomateGlobalGain => "Automate global gain + 1",
        AutomateGlobalExpGain => "Automate exp gain",
        ChildCostReduction => "Reduce cost of child upgrade if this bar has the upgrade",
    }
}
