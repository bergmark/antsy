use std::collections::HashMap;
use std::time::{Duration, Instant};
use strum::*;

use crate::app::{App, Completion};
use crate::float::Float;
use crate::prestige::{Prestige, PrestigeUpgrade};
use crate::upgrade::{GlobalUpgrade, Upgrade};

#[derive(Clone)]
pub(crate) struct Bar {
    pub(crate) progress: Float,
    pub(crate) gathered: Float,
    pub(crate) transfer_ratio: Float,
    pub(crate) last_completion: Option<Completion>,
    pub(crate) upgrades: HashMap<Upgrade, usize>,
    pub(crate) number: usize,
    pub(crate) exp: Float,
    pub(crate) level: usize,
    pub(crate) boost_until: Option<Instant>,
    pub(crate) gain_exponent: usize,
    pub(crate) level_speed: Float,
}

impl Bar {
    pub(crate) fn new(number: usize) -> Bar {
        Bar {
            progress: 0.0.into(),
            gathered: 0.0.into(),
            transfer_ratio: 0.01.into(),
            last_completion: None,
            upgrades: Upgrade::iter().map(|u| (u, 0)).collect(),
            number,
            exp: 0.0.into(),
            level: 1,
            boost_until: None,
            /// Slow down the progress bars. When progress finishes,
            /// exp and gains need to be incremented accordingly.
            gain_exponent: 0,
            level_speed: 1.0.into(),
        }
    }

    fn inc_exp(
        &mut self,
        global_exp_gain_levels: usize,
        global_exp_boost: usize,
        global_speed_levels: usize,
        prestige: &Prestige,
        now: Instant,
        next_bar: Option<&mut Bar>,
    ) {
        let mut exp_gain = Float(
            (1. + global_exp_gain_levels as f64)
                * 10usize.pow(self.gain_exponent as u32) as f64
                * 0.95_f64.powf(prestige.level_f(PrestigeUpgrade::LevelUpFaster)),
        );

        // Transfer exp
        if let Some(next_bar) = next_bar {
            if next_bar.level < self.level
                || (next_bar.level == self.level && next_bar.exp < self.exp)
            {
                let remaining = exp_gain * 0.99_f64.powf(prestige.level_f(PrestigeUpgrade::TransferExtraExp));
                let transfer = exp_gain - remaining;
                exp_gain = remaining;
                next_bar.exp += transfer;
            }
        }

        self.exp += exp_gain;
        self.check_level_up(global_exp_boost, global_speed_levels, now);
    }

    pub(crate) fn check_level_up(
        &mut self,
        global_exp_boost: usize,
        global_speed_levels: usize,
        now: Instant,
    ) {
        let exp_for_next_level = self.exp_for_next_level();
        if self.exp >= exp_for_next_level {
            self.exp -= exp_for_next_level;
            self.level += 1;

            self.level_speed += Float(0.01 * (self.level as f64 + 3.));

            self.adjust_speed_multiplier(global_speed_levels);

            let extra_dur = Duration::from_secs(1 + global_exp_boost as u64);
            match self.boost_until {
                None => self.boost_until = Some(now + extra_dur),
                Some(boost_until) => {
                    if boost_until < now {
                        self.boost_until = Some(now + extra_dur);
                    } else {
                        self.boost_until = Some(now + (boost_until - now) + extra_dur);
                    }
                }
            }
        }
    }

    pub(crate) fn adjust_speed_multiplier(&mut self, global_speed_levels: usize) {
        if self.speed_multiplier(global_speed_levels) >= 10. {
            self.gain_exponent += 1;
        }
    }

    pub(crate) fn is_boosted(&self, now: Instant) -> bool {
        self.boost_until.map_or(false, |until| until > now)
    }

    fn exp_for_level(level: usize) -> Float {
        Float(1.5).powf(level as f64)
    }

    pub(crate) fn exp_for_next_level(&self) -> Float {
        Self::exp_for_level(self.level + 1)
    }

    pub(crate) fn gain(&self, app: &App) -> Float {
        const GAIN_BASE: Float = Float(1.);
        use Upgrade::*;
        (GAIN_BASE + self.get_upgrade(Gain) + app.get_global_upgrade(GlobalUpgrade::Gain))
            * Float(2.).pow(self.get_upgrade(Double))
            * Float(3.).pow(self.get_upgrade(Triple))
            * Float(4.).pow(self.get_upgrade(Quadruple))
            * Float(10.0_f64.powf(self.gain_exponent as f64))
    }

    fn speed(&self, speed_base: Float, global_speed_levels: usize) -> Float {
        speed_base * self.speed_multiplier(global_speed_levels)
    }

    pub(crate) fn speed_multiplier(&self, global_speed_levels: usize) -> Float {
        Float(1.25).pow(self.get_upgrade(Upgrade::Speed))
            * Float(1.05).pow(global_speed_levels.into())
            * self.level_speed
            * Float(10.0_f64).pow(-Float::from(self.gain_exponent))
    }

    pub(crate) fn upgrade_cost(&self, upgrade: Upgrade) -> Float {
        let level = *self.upgrades.get(&upgrade).unwrap();
        upgrade.base_cost() * upgrade.scaling().powf(level as f64)
    }

    pub(crate) fn inc(
        &mut self,
        speed_base: Float,
        global_speed_levels: usize,
        global_exp_gain_levels: usize,
        global_exp_boost: usize,
        prestige: &Prestige,
        now: Instant,
        next_bar: Option<&mut Bar>,
    ) -> bool {
        let boost_mult = if self.boost_until.map_or(false, |until| now < until) {
            2.
        } else {
            1.
        };
        let new = self.progress + self.speed(speed_base, global_speed_levels) * boost_mult;
        if new > 100. * 0.95_f64.powf(prestige.level_f(PrestigeUpgrade::CompleteFaster)) {
            self.inc_exp(
                global_exp_gain_levels,
                global_exp_boost,
                global_speed_levels,
                prestige,
                now,
                next_bar,
            );
            self.progress = Float(100.) - self.progress;
            true
        } else {
            self.progress = new;
            false
        }
    }

    pub(crate) fn recent_completion(&self, now: Instant) -> Option<Completion> {
        self.last_completion
            .iter()
            .copied()
            .find(|c| now - c.tick < Duration::from_secs(1))
    }

    pub(crate) fn inc_upgrade(&mut self, upgrade: Upgrade, global_speed_levels: usize) {
        *self
            .upgrades
            .entry(upgrade)
            .or_insert_with(|| panic!("Should have been init'd")) += 1;
        if let Upgrade::Speed = upgrade {
            self.adjust_speed_multiplier(global_speed_levels);
        }
    }

    pub(crate) fn get_upgrade(&self, upgrade: Upgrade) -> Float {
        self.get_upgrade_u(upgrade).into()
    }

    pub(crate) fn get_upgrade_u(&self, upgrade: Upgrade) -> usize {
        self.upgrades[&upgrade]
    }
}
