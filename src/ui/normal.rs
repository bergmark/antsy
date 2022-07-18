use crate::upgrade::{GlobalUpgrade, Upgrade};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Highlight {
    None,
    Bar { upgrade: Upgrade, row: usize },
    Global { upgrade: GlobalUpgrade },
}

impl Highlight {
    pub(crate) fn new() -> Self {
        Highlight::None
    }
}
