use std::fmt::Display;

#[derive(PartialEq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}
impl TryFrom<&str> for Version {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.trim().split('.').collect();
        if parts.len() != 3 {
            return Err("Version must be in format 'major.minor.patch'");
        }
        Ok(Version {
            major: parts[0].parse().map_err(|_| "Invalid major version")?,
            minor: parts[1].parse().map_err(|_| "Invalid minor version")?,
            patch: parts[2].parse().map_err(|_| "Invalid patch version")?,
        })
    }
}
impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}