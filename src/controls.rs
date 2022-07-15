use crossterm::event::{KeyCode, KeyEvent};

use crate::upgrade::{GlobalUpgrade, Upgrade};

pub(crate) struct UiState {
    pub(crate) highlight: Option<Highlight>,
}

impl UiState {
    pub(crate) fn new() -> UiState {
        UiState { highlight: None }
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum Action {
    Quit,
    Noop,
    PurchaseUpgrade,
    UpgradeAny,
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum Highlight {
    Bar { upgrade: Upgrade, row: usize },
    Global { upgrade: GlobalUpgrade },
}

#[derive(Copy, Clone, Debug)]
enum Dir {
    Down,
    Left,
    Right,
    Up,
}

impl UiState {
    pub(crate) fn handle_keypress(&mut self, key: KeyEvent, bar_len: usize) -> Action {
        match key.code {
            KeyCode::Char('q') => return Action::Quit,
            KeyCode::Char('u') => return Action::UpgradeAny,
            KeyCode::Enter | KeyCode::Char(' ') => return Action::PurchaseUpgrade,
            KeyCode::Tab => self.change_highlight_pane(bar_len),
            KeyCode::Down => self.move_highlight(bar_len, Dir::Down),
            KeyCode::Up => self.move_highlight(bar_len, Dir::Up),
            KeyCode::Right => self.move_highlight(bar_len, Dir::Right),
            KeyCode::Left => self.move_highlight(bar_len, Dir::Left),
            _ => {}
        }

        Action::Noop
    }

    fn change_highlight_pane(&mut self, bar_len: usize) {
        match self.highlight {
            None => self.move_highlight(bar_len, Dir::Down),
            Some(Highlight::Global { .. }) => {
                self.highlight = Some(Highlight::Bar {
                    upgrade: Upgrade::Speed,
                    row: 0,
                })
            }
            Some(Highlight::Bar { .. }) => {
                self.highlight = Some(Highlight::Global {
                    upgrade: GlobalUpgrade::Speed,
                })
            }
        }
    }

    fn move_highlight(&mut self, bar_len: usize, dir: Dir) {
        if bar_len == 0 {
            return;
        }

        self.highlight = Some(match self.highlight {
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
                        row = bar_len - 1;
                    }
                    Highlight::Bar { row, upgrade }
                }
                Dir::Down => {
                    if row == bar_len - 1 {
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
}
