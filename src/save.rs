use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[derive(Serialize, Deserialize)]
pub(crate) struct App {
    bars: Vec<Bar>,
    bars_to_spawn: usize,
    last_bar_number: usize,
    global_upgrades: HashMap<GlobalUpgrade, usize>,
}

impl App {
    pub(crate) fn from_game(a: &crate::App) -> App {
        App {
            bars: a.bars.iter().map(|b| Bar::from_game(b, a.tick)).collect(),
            bars_to_spawn: a.bars_to_spawn,
            last_bar_number: a.last_bar_number,
            global_upgrades: a
                .global_upgrades
                .iter()
                .map(|(u, n)| (GlobalUpgrade::from_game(*u), *n))
                .collect(),
        }
    }

    pub(crate) fn into_game(self, now: Instant) -> crate::App {
        crate::App {
            bars: self.bars.into_iter().map(|b| b.into_game(now)).collect(),
            tick: now,
            last_bar_spawn: None,
            bars_to_spawn: self.bars_to_spawn,
            highlight: None,
            last_bar_number: self.last_bar_number,
            global_upgrades: self
                .global_upgrades
                .into_iter()
                .map(|(u, n)| (u.into_game(), n))
                .collect(),
            last_save: None,
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
}

impl Bar {
    fn from_game(b: &crate::bar::Bar, now: Instant) -> Bar {
        Bar {
            progress: b.progress.0,
            gathered: b.gathered.0,
            transfer_ratio: b.transfer_ratio.0,
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
            speed_base: b.speed_base.0,
            gain_exponent: b.gain_exponent,
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
    fn from_game(u: &crate::Upgrade) -> Upgrade {
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
