use crossterm::event::{KeyCode, KeyEvent};

use crate::controls::{Action, Dir, UiToggle};
use crate::ui;
use crate::ui::prestige::Highlight;

impl ui::Prestige {
    pub(super) fn handle_keypress(
        &mut self,
        key: KeyEvent,
        prestige_upgrade_len: usize,
    ) -> Result<Action, UiToggle> {
        match key.code {
            KeyCode::Char('p') => Err(UiToggle::ToNormal),
            KeyCode::Char('q') => Ok(Action::Quit),
            KeyCode::Enter | KeyCode::Char(' ') => self.purchase(),
            KeyCode::Down => self.move_highlight(prestige_upgrade_len, Dir::Down),
            KeyCode::Up => self.move_highlight(prestige_upgrade_len, Dir::Up),
            KeyCode::Right => self.move_highlight(prestige_upgrade_len, Dir::Right),
            KeyCode::Left => self.move_highlight(prestige_upgrade_len, Dir::Left),
            _ => Ok(Action::Noop),
        }
    }

    fn move_highlight(
        &mut self,
        prestige_upgrade_len: usize,
        dir: Dir,
    ) -> Result<Action, UiToggle> {
        use ui::prestige::Highlight::*;

        self.highlight = match self.highlight {
            None => Upgrade(0),
            Upgrade(mut row) => match dir {
                Dir::Left | Dir::Right => PrestigeButton,
                Dir::Down => Upgrade((row + 1) % prestige_upgrade_len),
                Dir::Up => {
                    if row > 0 {
                        row -= 1;
                    } else {
                        row = prestige_upgrade_len - 1;
                    }
                    Upgrade(row)
                }
            },
            PrestigeButton => match dir {
                Dir::Left | Dir::Right => Upgrade(0),
                Dir::Down | Dir::Up => PrestigeButton,
            },
        };

        Ok(Action::Noop)
    }

    fn purchase(&self) -> Result<Action, UiToggle> {
        Ok(match self.highlight {
            Highlight::None => Action::Noop,
            Highlight::PrestigeButton => Action::Prestige,
            Highlight::Upgrade(_) => Action::PurchasePrestigeUpgrade,
        })
    }
}
