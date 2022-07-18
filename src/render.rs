use tui::{backend::Backend, Frame};

use crate::app::App;
use crate::ui::UiState;

mod normal;
mod prestige;
mod util;

pub(crate) fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    match app.ui.state {
        UiState::Normal(n) => normal::render(f, app, n),
        UiState::Prestige(p) => prestige::render(f, app, p),
    }
}
