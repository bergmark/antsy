use tui::{backend::Backend, Frame};

use crate::app::App;
use crate::controls::UiStates;

mod normal;
mod prestige;
mod util;

pub(crate) fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    match app.ui_state.active {
        UiStates::Normal => normal::render(f, app),
        UiStates::Prestige => prestige::render(f, app),
    }
}
