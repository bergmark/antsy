use strum::*;

use crate::prestige::PrestigeUpgrade;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Highlight {
    None,
    PrestigeButton,
    Upgrade(usize),
}

impl Highlight {
    pub(crate) fn new() -> Self {
        Highlight::None
    }

    pub(crate) fn upgrade(self) -> Option<PrestigeUpgrade> {
        if let Highlight::Upgrade(i) = self {
            PrestigeUpgrade::iter().nth(i)
        } else {
            None
        }
    }
}
