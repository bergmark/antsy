use crossterm::event::KeyEvent;

use crate::ui::{Ui, UiState};

mod normal;
mod prestige;

#[derive(Debug, Copy, Clone)]
pub(crate) enum Action {
    Quit,
    Noop,
    PurchaseUpgrade,
    PurchasePrestigeUpgrade,
    Prestige,
    UpgradeAny,
}

enum UiToggle {
    ToNormal,
    ToPrestige,
}

#[derive(Copy, Clone, Debug)]
enum Dir {
    Down,
    Left,
    Right,
    Up,
}

impl Ui {
    pub(crate) fn handle_keypress(
        &mut self,
        key: KeyEvent,
        bar_len: usize,
        prestige_upgrade_len: usize,
    ) -> Action {
        let res = match &mut self.state {
            UiState::Normal(normal) => normal.handle_keypress(key, bar_len),
            UiState::Prestige(prestige) => prestige.handle_keypress(key, prestige_upgrade_len),
        };
        match res {
            Err(UiToggle::ToNormal) => {
                self.to_normal();
                Action::Noop
            }
            Err(UiToggle::ToPrestige) => {
                self.to_prestige();
                Action::Noop
            }
            Ok(action) => action,
        }
    }
}
