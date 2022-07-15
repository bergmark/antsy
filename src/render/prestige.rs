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

pub(crate) fn render<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = render_border(f, f.size(), "Prestige");
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(40), Constraint::Length(10)])
        .split(chunks);

    let left = chunks[0];
    let right = chunks[1];

    render_prestige_stats(f, app, left);
    render_prestige_upgrades(f, app, right);
}

fn render_prestige_stats<B: Backend>(f: &mut Frame<B>, app: &App, chunks: Rect) {
    if !app.prestige.can_prestige(app) {
        render_text(f, chunks, "You cannot prestige until you reach 10 bars");
    } else {
        let chunks = rect_to_lines(chunks);
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
                app.prestige.claimable_prestige(app)
            ),
        );
    }
}

fn render_prestige_upgrades<B: Backend>(f: &mut Frame<B>, app: &App, chunks: Rect) {
    let chunks = rect_to_lines(chunks);
    for (upgrade, chunk) in PrestigeUpgrade::iter().zip(chunks.into_iter()) {
        let cost = if app.prestige.is_max_level(upgrade) {
            "MAXED".to_owned()
        } else {
            app.prestige.cost(upgrade).to_string()
        };
        let text = format!("{label}: {cost}", label = prestige_upgrade_label(upgrade),);
        render_left_text(f, chunk, &text);
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
