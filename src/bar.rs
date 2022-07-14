use crate::app::{exp_for_level, App, Completion};
use crate::float::Float;
use crate::upgrade::{GlobalUpgrade, Upgrade};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use strum::*;

#[derive(Clone)]
pub(crate) struct Bar {
    pub(crate) progress: Float,
    pub(crate) gathered: Float,
    pub(crate) transfer_ratio: Float,
    pub(crate) last_completion: Option<Completion>,
    pub(crate) upgrades: HashMap<Upgrade, usize>,
    pub(crate) number: usize,
    pub(crate) exp: usize,
    pub(crate) level: usize,
    pub(crate) boost_until: Option<Instant>,
    pub(crate) speed_base: Float,
    pub(crate) gain_exponent: usize,
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
            exp: 0,
            level: 1,
            boost_until: None,
            speed_base: 1.0.into(),
            gain_exponent: 0,
        }
    }

    fn inc_exp(
        &mut self,
        global_exp_gain_levels: usize,
        global_exp_boost: usize,
        global_speed_levels: usize,
        now: Instant,
    ) {
        self.exp += (1 + global_exp_gain_levels) * 10usize.pow(self.gain_exponent as u32);
        let exp_for_next_level = self.exp_for_next_level();

        // Level up
        if self.exp >= exp_for_next_level {
            self.exp -= exp_for_next_level;
            self.level += 1;

            if self.speed(global_speed_levels) >= 10. {
                self.speed_base *= 0.1;
                self.gain_exponent += 1;
            }

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

    pub(crate) fn is_boosted(&self, now: Instant) -> bool {
        self.boost_until.map_or(false, |until| until > now)
    }

    pub(crate) fn exp_for_next_level(&self) -> usize {
        exp_for_level(self.level + 1)
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

    pub(crate) fn speed(&self, global_speed_levels: usize) -> Float {
        self.speed_base
            * Float(1.25).pow(self.get_upgrade(Upgrade::Speed))
            * Float(1.05).pow(global_speed_levels.into())
            * self.level_speed()
    }

    fn level_speed(&self) -> f64 {
        let mut level = self.level;
        let mut speed = 1f64;
        while level >= 2 {
            speed *= 1. + 0.01 * ((level + 4) as f64);
            level -= 1;
        }
        speed
    }

    pub(crate) fn upgrade_cost(&self, upgrade: Upgrade) -> Float {
        let level = *self.upgrades.get(&upgrade).unwrap();
        upgrade.base_cost() * upgrade.scaling().powf(level as f64)
    }

    pub(crate) fn inc(
        &mut self,
        global_speed_levels: usize,
        global_exp_gain_levels: usize,
        global_exp_boost: usize,
        now: Instant,
    ) -> bool {
        let boost_mult = if self.boost_until.map_or(false, |until| now < until) {
            2.
        } else {
            1.
        };
        let new = self.progress + self.speed(global_speed_levels) * boost_mult;
        if self.number == 1 {
            // log(&format!("progress: {new}"));
        }
        if new > 100. {
            self.inc_exp(
                global_exp_gain_levels,
                global_exp_boost,
                global_speed_levels,
                now,
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

    pub(crate) fn inc_upgrade(&mut self, upgrade: Upgrade) {
        *self
            .upgrades
            .entry(upgrade)
            .or_insert_with(|| panic!("Should have been init'd")) += 1;
    }
    pub(crate) fn get_upgrade(&self, upgrade: Upgrade) -> Float {
        self.get_upgrade_u(upgrade).into()
    }
    pub(crate) fn get_upgrade_u(&self, upgrade: Upgrade) -> usize {
        self.upgrades[&upgrade]
    }
}
