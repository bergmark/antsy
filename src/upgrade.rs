use crate::Float;
use strum::*;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, EnumIter, EnumCount, Hash)]
pub(crate) enum Upgrade {
    Speed,
    Gain,
    Double,
    Triple,
    Quadruple,
}

impl Upgrade {
    pub(crate) fn next(self) -> Upgrade {
        use Upgrade::*;
        match self {
            Speed => Gain,
            Gain => Double,
            Double => Triple,
            Triple => Quadruple,
            Quadruple => Speed,
        }
    }
    pub(crate) fn prev(self) -> Upgrade {
        use Upgrade::*;
        match self {
            Speed => Quadruple,
            Gain => Speed,
            Double => Gain,
            Triple => Double,
            Quadruple => Triple,
        }
    }
    pub(crate) fn base_cost(self) -> Float {
        use Upgrade::*;
        match self {
            Speed => 125.,
            Gain => 3.,
            Double => 200.,
            Triple => 5_000.,
            Quadruple => 100_000.,
        }
        .into()
    }
    pub(crate) fn cost_target(self) -> i64 {
        use Upgrade::*;
        match self {
            Speed => 0,
            Gain => 0,
            Double => 1,
            Triple => 4,
            Quadruple => 7,
        }
    }
    pub(crate) fn scaling(self) -> f64 {
        use Upgrade::*;
        match self {
            Speed => 5.,
            Gain => 2.,
            Double => 100.,
            Triple => 1_000.,
            Quadruple => 10_000.,
        }
    }
    pub(crate) fn cost(self, level: usize) -> Float {
        self.base_cost() * self.scaling().powf(level as f64)
    }

    pub const fn upgrade_preference_order() -> [Self; Self::COUNT] {
        use Upgrade::*;
        [Quadruple, Triple, Double, Gain, Speed]
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, EnumIter, EnumCount, Hash)]
pub(crate) enum GlobalUpgrade {
    Speed,
    ExpBoost,
    ProgressBars,
    Gain,
    ExpGain,
}

impl GlobalUpgrade {
    pub(crate) fn next(self) -> Self {
        use GlobalUpgrade::*;
        match self {
            Speed => ExpBoost,
            ExpBoost => ProgressBars,
            ProgressBars => Gain,
            Gain => ExpGain,
            ExpGain => Speed,
        }
    }
    pub(crate) fn prev(self) -> Self {
        use GlobalUpgrade::*;
        match self {
            Speed => ExpGain,
            ExpBoost => Speed,
            ProgressBars => ExpBoost,
            Gain => ProgressBars,
            ExpGain => Gain,
        }
    }
    fn base_cost(self) -> Float {
        use GlobalUpgrade::*;
        match self {
            Speed => 300.,
            ExpBoost => 30.,
            ProgressBars => 22.,
            Gain => 120.,
            ExpGain => 10_000.,
        }
        .into()
    }
    fn scaling(self) -> Float {
        use GlobalUpgrade::*;
        match self {
            Speed => 3.,
            ExpBoost => 1.5,
            ProgressBars => 3.5,
            Gain => 3.,
            ExpGain => 8.,
        }
        .into()
    }
    pub(crate) fn cost(self, level: usize) -> Float {
        self.base_cost() * self.scaling().powf(level as f64)
    }

    pub const fn upgrade_preference_order() -> [Self; Self::COUNT] {
        use GlobalUpgrade::*;
        [ProgressBars, Gain, Speed, ExpGain, ExpBoost]
    }
}
