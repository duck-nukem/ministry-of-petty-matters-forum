use std::fmt::{Display, Formatter};

pub struct Seconds(pub u32);
pub struct Minutes(pub u8);
pub struct Hours(pub u8);
pub struct Days(pub u8);

impl Display for Seconds {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait TimeUnit {
    fn to_seconds(&self) -> Seconds;
}

impl TimeUnit for Minutes {
    fn to_seconds(&self) -> Seconds {
        Seconds(self.0 as u32 * 60)
    }
}

impl TimeUnit for Hours {
    fn to_seconds(&self) -> Seconds {
        Seconds(self.0 as u32 * 3600)
    }
}

impl TimeUnit for Days {
    fn to_seconds(&self) -> Seconds {
        Seconds(self.0 as u32 * 86400)
    }
}