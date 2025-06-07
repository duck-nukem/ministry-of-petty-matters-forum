use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Seconds(pub u32);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Minutes(pub u16);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Hours(pub u16);

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Days(pub u16);

impl Display for Seconds {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} seconds", self.0)
    }
}

impl Display for Minutes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} minutes", self.0)
    }
}

impl Display for Hours {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} hours", self.0)
    }
}

impl Display for Days {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} days", self.0)
    }
}

impl From<Minutes> for Seconds {
    fn from(minutes: Minutes) -> Self {
        Self(u32::from(minutes.0 * 60))
    }
}

impl From<Hours> for Seconds {
    fn from(hours: Hours) -> Self {
        Self(u32::from(hours.0) * 3600)
    }
}

impl From<Days> for Seconds {
    fn from(days: Days) -> Self {
        Self(u32::from(days.0) * 86400)
    }
}

impl From<Hours> for Minutes {
    fn from(hours: Hours) -> Self {
        Self(hours.0 * 60)
    }
}

impl From<Days> for Minutes {
    fn from(days: Days) -> Self {
        Self(days.0 * 24 * 60)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_minutes_to_seconds() {
        let minute = Minutes(1);

        let minute_as_seconds: Seconds = minute.into();

        assert_eq!(minute_as_seconds, Seconds(60));
    }

    #[test]
    fn test_hours_to_seconds() {
        let hour = Hours(1);

        let hour_as_seconds: Seconds = hour.into();

        assert_eq!(hour_as_seconds, Seconds(3600));
    }

    #[test]
    fn test_days_to_seconds() {
        let day = Days(1);

        let day_as_seconds: Seconds = day.into();

        assert_eq!(day_as_seconds, Seconds(86400));
    }

    #[test]
    fn test_hours_to_minutes() {
        let hour = Hours(1);

        let hour_as_minutes: Minutes = hour.into();

        assert_eq!(hour_as_minutes, Minutes(60));
    }

    #[test]
    fn test_days_to_minutes() {
        let day = Days(1);

        let hour_as_minutes: Minutes = day.into();

        assert_eq!(hour_as_minutes, Minutes(24 * 60));
    }
}
