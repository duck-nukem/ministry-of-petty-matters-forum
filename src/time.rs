use std::fmt::{Display, Formatter};

pub struct Seconds(pub u16);

impl Display for Seconds {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}