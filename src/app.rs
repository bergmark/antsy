use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};
use strum::*;

use crate::bar::Bar;
use crate::float::Float;
use crate::upgrade::{GlobalUpgrade, Upgrade};

pub(crate) struct App {
    pub(crate) bars: VecDeque<Bar>,
    pub(crate) tick: Instant,
    pub(crate) last_bar_spawn: Option<Instant>,
    pub(crate) bars_to_spawn: usize,
    pub(crate) highlight: Option<Highlight>,
    pub(crate) last_bar_number: usize,
    pub(crate) global_upgrades: HashMap<GlobalUpgrade, usize>,
    pub(crate) last_save: Option<Instant>,
}

#[derive(Copy, Clone, Debug)]
pub(crate) enum Highlight {
    Bar { upgrade: Upgrade, row: usize },
    Global { upgrade: GlobalUpgrade },
}

struct UpgradeCost {
    target: usize,
    cost: Float,
}

impl App {
    fn new() -> App {
        App {
            bars: VecDeque::new(),
            tick: Instant::now(),
            last_bar_spawn: None,
            bars_to_spawn: 4,
            highlight: None,
            last_bar_number: 0,
            global_upgrades: GlobalUpgrade::iter().map(|g| (g, 0)).collect(),
            last_save: None,
        }
    }

    pub(crate) fn get_global_upgrade(&self, upgrade: GlobalUpgrade) -> Float {
        self.global_upgrades[&upgrade].into()
    }

    fn spawn_bar(&mut self) {
        self.last_bar_number += 1;
        self.bars.push_front(Bar::new(self.last_bar_number));
        if let Some(Highlight::Bar { row, upgrade: _ }) = &mut self.highlight {
            *row += 1;
        }
    }

    fn global_upgrade_price(&self, upgrade: GlobalUpgrade) -> Option<Float> {
        if self.bars.is_empty() {
            return None;
        }

        let level = self.global_upgrades[&upgrade];
        let cost = upgrade.cost(level);
        if self.bars[self.bars.len() - 1].gathered >= cost {
            Some(cost)
        } else {
            None
        }
    }

    pub(crate) fn can_afford_global(&self, upgrade: GlobalUpgrade) -> bool {
        self.global_upgrade_price(upgrade).is_some()
    }

    fn upgrade_price(&self, row: usize, upgrade: Upgrade) -> Option<UpgradeCost> {
        let cost = self.bars[row].upgrade_cost(upgrade);
        let target = row as i64 - upgrade.cost_target();
        if target >= 0 && self.bars[target as usize].gathered >= cost {
            Some(UpgradeCost {
                cost,
                target: target as usize,
            })
        } else {
            None
        }
    }

    pub(crate) fn can_afford(&self, row: usize, upgrade: Upgrade) -> bool {
        self.upgrade_price(row, upgrade).is_some()
    }

    pub(crate) fn highlight_cost_target(&self) -> Option<i64> {
        match self.highlight {
            Some(Highlight::Bar { row, upgrade }) => {
                let target = row as i64 - upgrade.cost_target();
                Some(target)
            }
            Some(Highlight::Global { .. }) => {
                if self.bars.is_empty() {
                    None
                } else {
                    Some((self.bars.len() - 1) as i64)
                }
            }
            None => None,
        }
    }

    pub(crate) fn purchase_upgrade(&mut self) {
        if let Some(Highlight::Bar { upgrade, row }) = self.highlight {
            if let Some(upgrade_cost) = self.upgrade_price(row, upgrade) {
                self.bars[row].inc_upgrade(upgrade);
                self.bars[upgrade_cost.target].gathered -= upgrade_cost.cost;
            }
        }
        if let Some(Highlight::Global { upgrade }) = self.highlight {
            if let Some(upgrade_cost) = self.global_upgrade_price(upgrade) {
                *self
                    .global_upgrades
                    .entry(upgrade)
                    .or_insert_with(|| panic!("Should have been init'd")) += 1;
                let last_i = self.bars.len() - 1;
                self.bars[last_i].gathered -= upgrade_cost;
                if let GlobalUpgrade::ProgressBars = upgrade {
                    self.bars_to_spawn += 2;
                }
            }
        }
    }

    pub(crate) fn on_tick(&mut self, now: Instant) {
        self.tick = now;

        if self.bars_to_spawn > 0
            && (self.last_bar_spawn.is_none()
                || now - self.last_bar_spawn.unwrap() >= Duration::from_secs(1))
        {
            self.spawn_bar();
            self.bars_to_spawn -= 1;
            self.last_bar_spawn = Some(self.tick);
        }

        for i in 0..self.bars.len() {
            let read_bar = self.bars.get(i).unwrap().clone();
            let done = self.bars[i].inc(
                self.global_upgrades[&GlobalUpgrade::Speed],
                self.global_upgrades[&GlobalUpgrade::ExpGain],
                self.global_upgrades[&GlobalUpgrade::ExpBoost],
                now,
            );
            if done {
                let gain = read_bar.gain(self);
                self.bars[i].gathered += gain;

                if i + 1 < self.bars.len() {
                    let transferred = self.bars[i].gathered * self.bars[i].transfer_ratio;
                    let gained = read_bar.gain(self) - transferred;
                    self.bars[i + 1].gathered += transferred;
                    self.bars[i].gathered -= transferred;
                    self.bars[i].last_completion = Some(Completion {
                        gain: gained,
                        transferred: Some(transferred),
                        tick: now,
                    });
                } else {
                    self.bars[i].last_completion = Some(Completion {
                        gain: read_bar.gain(self),
                        transferred: None,
                        tick: now,
                    });
                }
            }
        }

        match self.last_save {
            None => self.save(),
            Some(last_save) => {
                if self.tick - last_save > Duration::from_secs(30) {
                    self.save()
                }
            }
        }
    }

    pub(crate) fn load(now: Instant) -> App {
        let path = std::path::Path::new("save.json");
        if !path.exists() {
            App::new()
        } else {
            let contents = std::fs::read_to_string(&path).unwrap();
            let save: crate::save::App = serde_json::from_str(&contents).unwrap();
            save.into_game(now)
        }
    }

    pub(crate) fn save(&self) {
        let save = crate::save::App::from_game(self);
        std::fs::write("save.json", serde_json::to_string(&save).unwrap()).unwrap();
    }
}

#[derive(Copy, Clone)]
pub(crate) struct Completion {
    pub(crate) gain: Float,
    pub(crate) transferred: Option<Float>,
    pub(crate) tick: Instant,
}

pub(crate) fn exp_for_level(level: usize) -> usize {
    1.5f64.powf((level - 1) as f64) as usize
}
