use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::controls::UiState;
use crate::opts::Opts;

#[derive(Serialize, Deserialize)]
pub(crate) struct App {
    bars: Vec<Bar>,
    bars_to_spawn: usize,
    last_bar_number: usize,
    global_upgrades: HashMap<GlobalUpgrade, usize>,
    // Doesn't exist in old saves
    prestige: Option<Prestige>,
}

impl App {
    pub(crate) fn from_game(a: &crate::app::App) -> App {
        App {
            bars: a.bars.iter().map(|b| Bar::from_game(b, a.tick)).collect(),
            bars_to_spawn: a.bars_to_spawn,
            last_bar_number: a.last_bar_number,
            global_upgrades: a
                .global_upgrades
                .iter()
                .map(|(u, n)| (GlobalUpgrade::from_game(*u), *n))
                .collect(),
            prestige: Some(Prestige::from_game(&a.prestige)),
        }
    }

    pub(crate) fn into_game(self, opts: Opts, now: Instant) -> crate::app::App {
        crate::app::App {
            bars: self.bars.into_iter().map(|b| b.into_game(now)).collect(),
            tick: now,
            last_bar_spawn: None,
            bars_to_spawn: self.bars_to_spawn,
            ui_state: UiState::new(opts.start_state),
            last_bar_number: self.last_bar_number,
            global_upgrades: self
                .global_upgrades
                .into_iter()
                .map(|(u, n)| (u.into_game(), n))
                .collect(),
            last_save: None,
            opts,
            prestige: self
                .prestige
                .map_or(crate::prestige::Prestige::new(), Prestige::into_game),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone)]
enum GlobalUpgrade {
    Speed,
    ExpBoost,
    ProgressBars,
    Gain,
    ExpGain,
}

impl GlobalUpgrade {
    fn from_game(u: crate::GlobalUpgrade) -> GlobalUpgrade {
        use GlobalUpgrade::*;
        match u {
            crate::GlobalUpgrade::Speed => Speed,
            crate::GlobalUpgrade::ExpBoost => ExpBoost,
            crate::GlobalUpgrade::ProgressBars => ProgressBars,
            crate::GlobalUpgrade::Gain => Gain,
            crate::GlobalUpgrade::ExpGain => ExpGain,
        }
    }
    fn into_game(self) -> crate::GlobalUpgrade {
        use GlobalUpgrade::*;
        match self {
            Speed => crate::GlobalUpgrade::Speed,
            ExpBoost => crate::GlobalUpgrade::ExpBoost,
            ProgressBars => crate::GlobalUpgrade::ProgressBars,
            Gain => crate::GlobalUpgrade::Gain,
            ExpGain => crate::GlobalUpgrade::ExpGain,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Bar {
    progress: f64,
    gathered: f64,
    transfer_ratio: f64,
    upgrades: HashMap<Upgrade, usize>,
    number: usize,
    exp: usize,
    level: usize,
    boost_remaining: Duration,
    speed_base: f64,
    gain_exponent: usize,
    level_speed: f64,
}

impl Bar {
    fn from_game(b: &crate::bar::Bar, now: Instant) -> Bar {
        Bar {
            progress: b.progress.into(),
            gathered: b.gathered.into(),
            transfer_ratio: b.transfer_ratio.into(),
            upgrades: b
                .upgrades
                .iter()
                .map(|(u, n)| (Upgrade::from_game(u), *n))
                .collect(),
            number: b.number,
            exp: b.exp,
            level: b.level,
            boost_remaining: b.boost_until.map_or(Duration::from_secs(0), |until| {
                if until > now {
                    until - now
                } else {
                    Duration::from_secs(0)
                }
            }),
            speed_base: b.speed_base.into(),
            gain_exponent: b.gain_exponent,
            level_speed: b.level_speed.into(),
        }
    }

    fn into_game(self, now: Instant) -> crate::Bar {
        crate::Bar {
            progress: self.progress.into(),
            gathered: self.gathered.into(),
            transfer_ratio: self.transfer_ratio.into(),
            last_completion: None,
            upgrades: self
                .upgrades
                .into_iter()
                .map(|(u, n)| (u.into_game(), n))
                .collect(),
            number: self.number,
            exp: self.exp,
            level: self.level,
            boost_until: Some(now + self.boost_remaining),
            speed_base: self.speed_base.into(),
            gain_exponent: self.gain_exponent,
            level_speed: self.level_speed.into(),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone)]
pub enum Upgrade {
    Speed,
    Gain,
    Double,
    Triple,
    Quadruple,
}

impl Upgrade {
    fn from_game(u: &crate::Upgrade) -> Self {
        use Upgrade::*;
        match u {
            crate::Upgrade::Speed => Speed,
            crate::Upgrade::Gain => Gain,
            crate::Upgrade::Double => Double,
            crate::Upgrade::Triple => Triple,
            crate::Upgrade::Quadruple => Quadruple,
        }
    }

    fn into_game(self) -> crate::Upgrade {
        use Upgrade::*;
        match self {
            Speed => crate::Upgrade::Speed,
            Gain => crate::Upgrade::Gain,
            Double => crate::Upgrade::Double,
            Triple => crate::Upgrade::Triple,
            Quadruple => crate::Upgrade::Quadruple,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Prestige {
    current: f64,
    // Doesn't exist in old saves
    upgrades: Option<HashMap<PrestigeUpgrade, usize>>,
}

impl Prestige {
    fn from_game(p: &crate::prestige::Prestige) -> Self {
        Self {
            current: p.current.into(),
            upgrades: Some(
                p.upgrades
                    .iter()
                    .map(|(u, n)| (PrestigeUpgrade::from_game(u), *n))
                    .collect(),
            ),
        }
    }
    fn into_game(self) -> crate::prestige::Prestige {
        let Self { current, upgrades } = self;
        crate::prestige::Prestige {
            current: current.into(),
            upgrades: upgrades.map_or_else(
                || HashMap::new(),
                |upgrades| {
                    upgrades
                        .into_iter()
                        .map(|(u, n)| (u.into_game(), n))
                        .collect()
                },
            ),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
enum PrestigeUpgrade {
    CompleteFaster,
    LevelUpFaster,
    TransferExtraExp,
    TransferExtraValue,
    UpgradeAnyButton,
    AutomateGlobalSpeed,
    AutomateGlobalExpBoost,
    AutomateProgressBars,
    AutomateGlobalGain,
    AutomateGlobalExpGain,
    ChildCostReduction,
}

impl PrestigeUpgrade {
    fn from_game(u: &crate::prestige::PrestigeUpgrade) -> Self {
        use crate::prestige::PrestigeUpgrade as Game;
        use PrestigeUpgrade::*;
        match u {
            Game::CompleteFaster => CompleteFaster,
            Game::LevelUpFaster => LevelUpFaster,
            Game::TransferExtraExp => TransferExtraExp,
            Game::TransferExtraValue => TransferExtraValue,
            Game::UpgradeAnyButton => UpgradeAnyButton,
            Game::AutomateGlobalSpeed => AutomateGlobalSpeed,
            Game::AutomateGlobalExpBoost => AutomateGlobalExpBoost,
            Game::AutomateProgressBars => AutomateProgressBars,
            Game::AutomateGlobalGain => AutomateGlobalGain,
            Game::AutomateGlobalExpGain => AutomateGlobalExpGain,
            Game::ChildCostReduction => ChildCostReduction,
        }
    }

    fn into_game(self) -> crate::prestige::PrestigeUpgrade {
        use crate::prestige::PrestigeUpgrade as Game;
        use PrestigeUpgrade::*;
        match self {
            CompleteFaster => Game::CompleteFaster,
            LevelUpFaster => Game::LevelUpFaster,
            TransferExtraExp => Game::TransferExtraExp,
            TransferExtraValue => Game::TransferExtraValue,
            UpgradeAnyButton => Game::UpgradeAnyButton,
            AutomateGlobalSpeed => Game::AutomateGlobalSpeed,
            AutomateGlobalExpBoost => Game::AutomateGlobalExpBoost,
            AutomateProgressBars => Game::AutomateProgressBars,
            AutomateGlobalGain => Game::AutomateGlobalGain,
            AutomateGlobalExpGain => Game::AutomateGlobalExpGain,
            ChildCostReduction => Game::ChildCostReduction,
        }
    }
}
