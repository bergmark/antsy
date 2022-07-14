use format_num::NumberFormat;
use std::cmp;
use std::fmt;
use std::ops::Mul;

#[derive(Copy, Clone, PartialEq, PartialOrd, Add, AddAssign, Sub, SubAssign, MulAssign)]
pub struct Float(pub f64);

impl fmt::Display for Float {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", NumberFormat::new().format(".3s", (self.0 * 100.).floor() / 100.))
    }
}

impl Float {
    pub(crate) fn pow(self, f: Self) -> Self {
        self.0.powf(f.0).into()
    }
    pub(crate) fn powf(self, f: f64) -> Self {
        self.0.powf(f).into()
    }
}

impl From<f64> for Float {
    fn from(s: f64) -> Self {
        Self(s)
    }
}

impl From<Float> for f64 {
    fn from(s: Float) -> Self {
        s.0
    }
}

impl From<usize> for Float {
    fn from(s: usize) -> Self {
        Self(s as f64)
    }
}

impl Mul<Float> for Float {
    type Output = Self;
    fn mul(self, f: Self) -> Float {
        self * f.0
    }
}

impl Mul<f64> for Float {
    type Output = Self;
    fn mul(self, f: f64) -> Self {
        (self.0 * f).into()
    }
}

impl PartialEq<f64> for Float {
    fn eq(&self, f: &f64) -> bool {
        self.0.eq(f)
    }
}
impl PartialEq<Float> for f64 {
    fn eq(&self, f: &Float) -> bool {
        self.eq(&f.0)
    }
}

impl PartialOrd<f64> for Float {
    fn partial_cmp(&self, f: &f64) -> Option<cmp::Ordering> {
        self.0.partial_cmp(f)
    }
}

impl PartialOrd<Float> for f64 {
    fn partial_cmp(&self, f: &Float) -> Option<cmp::Ordering> {
        self.partial_cmp(&f.0)
    }
}
