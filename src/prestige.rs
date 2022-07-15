use crate::app::App;
use crate::float::Float;
use std::collections::HashMap;
use strum::*;

pub(crate) struct Prestige {
    pub(crate) current: Float,
    pub(crate) upgrades: HashMap<PrestigeUpgrade, usize>,
}

impl Prestige {
    pub(crate) fn new() -> Self {
        Self {
            current: 0.into(),
            upgrades: HashMap::new(),
        }
    }

    pub(crate) fn can_prestige(&self, app: &App) -> bool {
        app.bars.len() >= 10
    }

    pub(crate) fn claimable_prestige(&self, app: &App) -> Float {
        if !self.can_prestige(app) {
            0.into()
        } else {
            Float((app.bars.len() as f64) / 10.)
        }
    }

    pub(crate) fn cost(&self, upgrade: PrestigeUpgrade) -> Float {
        Float::from(2_usize.pow(self.get_level(upgrade) as u32))
    }

    fn get_level(&self, upgrade: PrestigeUpgrade) -> usize {
        *self.upgrades.get(&upgrade).unwrap_or(&0_usize)
    }

    pub(crate) fn is_max_level(&self, upgrade: PrestigeUpgrade) -> bool {
        upgrade
            .max_level()
            .map_or(false, |max| self.get_level(upgrade) >= max)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, EnumIter, EnumCount, Hash)]
pub(crate) enum PrestigeUpgrade {
    CompleteFaster,     // progress needed 0.95*n
    LevelUpFaster,      // exp req 0.95*n
    TransferExtraExp,   // If bar below has less exp, transfer 0.01*n
    TransferExtraValue, // If bar below has less value, transfer 0.01*n
    UpgradeAnyButton, // level 1: upgrade bars, 2: speed, 3: x2, 4: x3, 5: x3, 6: global, 7: try upgrading everything one time, 8: upgrade everything multiple times

    // upgrade every 60s/n
    AutomateGlobalSpeed,
    AutomateGlobalExpBoost,
    AutomateProgressBars,
    AutomateGlobalGain,
    AutomateGlobalExpGain,

    ChildCostReduction, // If bar above has fewer upgrades, discount it by 1% per additional upgrade. level: number of affected children
}

impl PrestigeUpgrade {
    fn max_level(self) -> Option<usize> {
        use PrestigeUpgrade::*;
        match self {
            CompleteFaster => None,
            LevelUpFaster => None,
            TransferExtraExp => None,
            TransferExtraValue => None,
            UpgradeAnyButton => Some(8),
            AutomateGlobalSpeed => None,
            AutomateGlobalExpBoost => None,
            AutomateProgressBars => None,
            AutomateGlobalGain => None,
            AutomateGlobalExpGain => None,
            ChildCostReduction => None,
        }
    }
}
