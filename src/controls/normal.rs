use crossterm::event::{KeyCode, KeyEvent};

use crate::controls::{Action, Dir, UiToggle};
use crate::ui::{self};
use crate::upgrade::{GlobalUpgrade, Upgrade};

impl ui::Normal {
    pub(super) fn handle_keypress(
        &mut self,
        key: KeyEvent,
        bar_len: usize,
    ) -> Result<Action, UiToggle> {
        match key.code {
            KeyCode::Char('p') => return Err(UiToggle::ToPrestige),
            KeyCode::Char('q') => return Ok(Action::Quit),
            KeyCode::Char('u') => return Ok(Action::UpgradeAny),
            KeyCode::Enter | KeyCode::Char(' ') => return Ok(Action::PurchaseUpgrade),
            KeyCode::Tab => self.change_highlight_pane(bar_len),
            KeyCode::Down => self.move_highlight(bar_len, Dir::Down),
            KeyCode::Up => self.move_highlight(bar_len, Dir::Up),
            KeyCode::Right => self.move_highlight(bar_len, Dir::Right),
            KeyCode::Left => self.move_highlight(bar_len, Dir::Left),
            _ => (),
        };
        Ok(Action::Noop)
    }

    fn change_highlight_pane(&mut self, bar_len: usize) {
        use ui::normal::Highlight::*;
        match self.highlight {
            None => self.move_highlight(bar_len, Dir::Down),
            Global { .. } => {
                self.highlight = Bar {
                    upgrade: Upgrade::Speed,
                    row: 0,
                }
            }
            Bar { .. } => {
                self.highlight = Global {
                    upgrade: GlobalUpgrade::Speed,
                }
            }
        }
    }

    fn move_highlight(&mut self, bar_len: usize, dir: Dir) {
        use ui::normal::Highlight::*;
        if bar_len == 0 {
            return;
        }

        self.highlight = match self.highlight {
            None => Bar {
                upgrade: Upgrade::Speed,
                row: 0,
            },
            ui::normal::Highlight::Bar { upgrade, mut row } => match dir {
                Dir::Left => ui::normal::Highlight::Bar {
                    row,
                    upgrade: upgrade.prev(),
                },
                Dir::Right => ui::normal::Highlight::Bar {
                    row,
                    upgrade: upgrade.next(),
                },
                Dir::Up => {
                    if row > 0 {
                        row -= 1;
                    } else {
                        row = bar_len - 1;
                    }
                    ui::normal::Highlight::Bar { row, upgrade }
                }
                Dir::Down => {
                    if row == bar_len - 1 {
                        row = 0;
                    } else {
                        row += 1;
                    }
                    ui::normal::Highlight::Bar { row, upgrade }
                }
            },
            ui::normal::Highlight::Global { upgrade } => match dir {
                Dir::Left => ui::normal::Highlight::Global {
                    upgrade: upgrade.prev(),
                },
                Dir::Right => ui::normal::Highlight::Global {
                    upgrade: upgrade.next(),
                },
                Dir::Down | Dir::Up => ui::normal::Highlight::Global { upgrade },
            },
        };
    }
}
