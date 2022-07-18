use strum::*;

pub(crate) mod normal;
pub(crate) mod prestige;

use crate::prestige::PrestigeUpgrade;

#[derive(EnumString, Copy, Clone)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum UiStates {
    Normal,
    Prestige,
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct Ui {
    pub(crate) state: UiState,
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum UiState {
    Normal(Normal),
    Prestige(Prestige),
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct Normal {
    pub(crate) highlight: normal::Highlight,
}

#[derive(Copy, Clone, Debug)]
pub(crate) struct Prestige {
    pub(crate) highlight: prestige::Highlight,
}

impl Ui {
    pub(crate) fn new(active: Option<UiStates>) -> Self {
        Self {
            state: match active {
                None | Some(UiStates::Normal) => UiState::Normal(Normal::new()),
                Some(UiStates::Prestige) => UiState::Prestige(Prestige::new()),
            },
        }
    }

    pub(crate) fn to_normal(&mut self) {
        self.state = UiState::Normal(Normal::new());
    }

    pub(crate) fn to_prestige(&mut self) {
        self.state = UiState::Prestige(Prestige::new());
    }

    // pub(crate) fn tag(&self) -> UiStates {
    //     match self.state {
    //         UiState::Normal { .. } => UiStates::Normal,
    //         UiState::Prestige { .. } => UiStates::Prestige,
    //     }
    // }

    pub(crate) fn normal_highlight(&self) -> Option<normal::Highlight> {
        match self.state {
            UiState::Normal(u) => Some(u.highlight),
            _ => None,
        }
    }

    pub(crate) fn highlighted_prestige_upgrade(&self) -> Option<PrestigeUpgrade> {
        match self.state {
            UiState::Prestige(u) => u.highlight.upgrade(),
            _ => None,
        }
    }
}

impl Normal {
    pub(crate) fn new() -> Self {
        Self {
            highlight: normal::Highlight::new(),
        }
    }
}

impl Prestige {
    pub(crate) fn new() -> Self {
        Self {
            highlight: prestige::Highlight::new(),
        }
    }
}
