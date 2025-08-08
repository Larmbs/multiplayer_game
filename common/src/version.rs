use std::cmp::Ordering;
use std::fmt::Display;

#[derive(PartialEq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}
impl From<&str> for Version {
    fn from(value: &str) -> Self {
        Version::try_from(value).unwrap()
    }
}
impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        match self.major.cmp(&other.major) {
            Ordering::Equal => match self.minor.cmp(&other.minor) {
                Ordering::Equal => Some(self.patch.cmp(&other.patch)),
                ord => Some(ord),
            },
            ord => Some(ord),
        }
    }
}
