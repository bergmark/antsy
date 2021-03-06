use std::{
    collections::{HashMap, VecDeque},
    time::{Duration, Instant},
};
use strum::*;

use crate::bar::Bar;
use crate::float::Float;
use crate::opts::Opts;
use crate::prestige::{Prestige, PrestigeUpgrade};
use crate::ui::{self, Ui, UiState};
use crate::upgrade::{GlobalUpgrade, Upgrade};

pub(crate) struct App {
    pub(crate) bars: VecDeque<Bar>,
    pub(crate) tick: Instant,
    pub(crate) last_bar_spawn: Option<Instant>,
    pub(crate) bars_to_spawn: usize,
    pub(crate) last_bar_number: usize,
    pub(crate) global_upgrades: HashMap<GlobalUpgrade, usize>,
    pub(crate) last_save: Option<Instant>,
    pub(crate) opts: Opts,
    pub(crate) ui: Ui,
    pub(crate) prestige: Prestige,
    pub(crate) last_automation: HashMap<GlobalUpgrade, Instant>,
}

struct UpgradeCost {
    target: usize,
    cost: Float,
}

impl App {
    pub(crate) fn load(opts: Opts, now: Instant) -> App {
        let save = std::path::PathBuf::from(&opts.save_file);
        let mut app = if !save.exists() {
            App::new(opts)
        } else {
            let contents = std::fs::read_to_string(&opts.save_file).unwrap();
            let save: crate::save::App = serde_json::from_str(&contents).unwrap();
            save.into_game(opts, now)
        };

        let global_speed_levels = app.get_global_upgrade_u(GlobalUpgrade::Speed);
        for bar in &mut app.bars {
            bar.adjust_speed_multiplier(global_speed_levels);
        }

        app
    }

    pub(crate) fn save(&self) {
        let save = crate::save::App::from_game(self);
        std::fs::write(
            &self.opts.save_file,
            serde_json::to_string_pretty(&save).unwrap(),
        )
        .unwrap();
    }

    fn new(opts: Opts) -> App {
        App {
            bars: VecDeque::new(),
            tick: Instant::now(),
            last_bar_spawn: None,
            bars_to_spawn: 4,
            ui: Ui::new(opts.start_state),
            last_bar_number: 0,
            global_upgrades: GlobalUpgrade::iter().map(|g| (g, 0)).collect(),
            last_save: None,
            opts,
            prestige: Prestige::new(),
            last_automation: HashMap::new(),
        }
    }

    pub(crate) fn prestige(&mut self) {
        self.save();

        self.prestige.prestige(self.bars.len());

        self.bars = VecDeque::new();
        self.last_bar_spawn = None;
        self.last_bar_number = 0;
        self.bars_to_spawn = 4;
        self.global_upgrades = GlobalUpgrade::iter().map(|g| (g, 0)).collect();
        self.ui.to_normal();
    }

    pub(crate) fn get_global_upgrade(&self, upgrade: GlobalUpgrade) -> Float {
        self.get_global_upgrade_u(upgrade).into()
    }
    pub(crate) fn get_global_upgrade_u(&self, upgrade: GlobalUpgrade) -> usize {
        self.global_upgrades[&upgrade]
    }

    fn spawn_bar(&mut self) {
        self.last_bar_number += 1;
        self.bars.push_front(Bar::new(self.last_bar_number));
        if let UiState::Normal(ui::Normal {
            highlight: ui::normal::Highlight::Bar { row, upgrade: _ },
        }) = &mut self.ui.state
        {
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
        match self.ui.normal_highlight() {
            Some(ui::normal::Highlight::Bar { row, upgrade }) => {
                let target = row as i64 - upgrade.cost_target();
                Some(target)
            }
            Some(ui::normal::Highlight::Global { .. }) => {
                if self.bars.is_empty() {
                    None
                } else {
                    Some((self.bars.len() - 1) as i64)
                }
            }
            Some(ui::normal::Highlight::None) => None,
            None => None,
        }
    }

    pub(crate) fn try_purchase_highlighted_upgrade(&mut self) {
        if let Some(highlight) = self.ui.normal_highlight() {
            self.try_purchase_upgrade(highlight);
        }
    }

    fn try_purchase_upgrade(&mut self, highlight: ui::normal::Highlight) -> bool {
        match highlight {
            ui::normal::Highlight::None => {}
            ui::normal::Highlight::Bar { upgrade, row } => {
                if let Some(upgrade_cost) = self.upgrade_price(row, upgrade) {
                    let global_speed_levels = self.get_global_upgrade_u(GlobalUpgrade::Speed);
                    self.bars[row].inc_upgrade(upgrade, global_speed_levels);
                    self.bars[upgrade_cost.target].gathered -= upgrade_cost.cost;
                    return true;
                }
            }
            ui::normal::Highlight::Global { upgrade } => {
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
                    if let GlobalUpgrade::Speed = upgrade {
                        let global_speed_levels = self.get_global_upgrade_u(GlobalUpgrade::Speed);
                        for bar in &mut self.bars {
                            bar.adjust_speed_multiplier(global_speed_levels);
                        }
                    }
                    return true;
                }
            }
        }
        false
    }

    pub(crate) fn purchase_any_upgrade(&mut self) {
        for upgrade in GlobalUpgrade::upgrade_preference_order() {
            if self.try_purchase_upgrade(ui::normal::Highlight::Global { upgrade }) {
                return;
            }
        }
        let bar_len = self.bars.len();
        for upgrade in Upgrade::upgrade_preference_order() {
            for row in (0..bar_len).rev() {
                if self.try_purchase_upgrade(ui::normal::Highlight::Bar { upgrade, row }) {
                    return;
                }
            }
        }
    }

    fn speed_base(&self) -> Float {
        Float(self.opts.speed_base)
    }

    pub(crate) fn on_tick(&mut self, now: Instant) {
        self.tick = now;

        if self.bars_to_spawn > 0
            && (self.last_bar_spawn.map_or(true, |last_bar_spawn| {
                now - last_bar_spawn >= Duration::from_secs(1)
            }))
        {
            self.spawn_bar();
            self.bars_to_spawn -= 1;
            self.last_bar_spawn = Some(self.tick);
        }

        for i in 0..self.bars.len() {
            let read_bar = self.bars.get(i).unwrap().clone();
            let speed_base = self.speed_base();
            let (bar, next_bars) = self.bars.make_contiguous().split_at_mut(i + 1);
            let done = bar[bar.len() - 1].inc(
                speed_base,
                self.global_upgrades[&GlobalUpgrade::Speed],
                self.global_upgrades[&GlobalUpgrade::ExpGain],
                self.global_upgrades[&GlobalUpgrade::ExpBoost],
                &self.prestige,
                now,
                next_bars.get_mut(0),
            );
            if done {
                let gain = read_bar.gain(self);
                self.bars[i].gathered += gain;

                if i + 1 < self.bars.len() {
                    let mut transfer_ratio = self.bars[i].transfer_ratio;

                    if self.bars[i + 1].gathered < self.bars[i].gathered {
                        transfer_ratio += Float(0.01)
                            * self.prestige.level_f(PrestigeUpgrade::TransferExtraValue);
                    }

                    let transferred = self.bars[i].gathered * transfer_ratio;
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

        self.auto_purchase(now);

        match self.last_save {
            None => self.save(),
            Some(last_save) => {
                if self.tick - last_save > Duration::from_secs(30) {
                    self.save()
                }
            }
        }
    }

    fn auto_purchase(&mut self, now: Instant) {
        const PAIRS: [(PrestigeUpgrade, GlobalUpgrade); 5] = [
            (PrestigeUpgrade::AutomateGlobalSpeed, GlobalUpgrade::Speed),
            (PrestigeUpgrade::AutomateGlobalExpBoost, GlobalUpgrade::ExpBoost),
            (PrestigeUpgrade::AutomateProgressBars, GlobalUpgrade::ProgressBars),
            (PrestigeUpgrade::AutomateGlobalGain, GlobalUpgrade::Gain),
            (PrestigeUpgrade::AutomateGlobalExpGain, GlobalUpgrade::ExpGain),
        ];
        for (prestige, global) in PAIRS {
            let prestige_level = self.prestige.level(prestige);
            if prestige_level != 0 {
                let last_automation = *self.last_automation.entry(global).or_insert(now);
                let automation_interval = (60_000f64 / prestige_level as f64) as u64;
                let automation_interval = std::cmp::max(1, automation_interval);
                let automation_interval = Duration::from_millis(automation_interval);
                if now < last_automation + automation_interval {
                    self.try_purchase_upgrade(ui::normal::Highlight::Global { upgrade: global });
                    *self.last_automation.entry(global).or_insert(now) = now;
                }
            }
        }
    }
}

#[derive(Copy, Clone)]
pub(crate) struct Completion {
    pub(crate) gain: Float,
    pub(crate) transferred: Option<Float>,
    pub(crate) tick: Instant,
}
