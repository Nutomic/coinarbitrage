use std::fmt::{Display, Formatter};
use std::ops::Mul;
use tabled::Tabled;
use thousands::Separable;

#[derive(Debug, Tabled, PartialOrd, PartialEq, Copy, Clone)]
pub struct Euro(pub f32);

impl Display for Euro {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} €", self.0.separate_with_spaces())
    }
}

impl Mul for Euro {
    type Output = Euro;

    fn mul(self, rhs: Self) -> Self::Output {
        Euro(self.0.mul(rhs.0))
    }
}

#[derive(Debug, Tabled, PartialOrd, PartialEq, Copy, Clone)]
pub struct Percent(pub f32);

impl Display for Percent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2} %", self.0)
    }
}
