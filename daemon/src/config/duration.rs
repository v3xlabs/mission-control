use std::{fmt, time::Duration};

use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HumanDuration(pub Duration);

impl HumanDuration {
    pub fn seconds(&self) -> u64 {
        self.0.as_secs()
    }
}

impl From<HumanDuration> for Duration {
    fn from(value: HumanDuration) -> Self {
        value.0
    }
}

impl fmt::Display for HumanDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let millis = self.0.as_millis();

        if millis == 0 || !millis.is_multiple_of(1000) {
            return write!(f, "{millis}ms");
        }

        let total = self.0.as_secs();

        if total.is_multiple_of(3600) {
            write!(f, "{}h", total / 3600)
        } else if total.is_multiple_of(60) {
            write!(f, "{}m", total / 60)
        } else {
            write!(f, "{total}s")
        }
    }
}

impl HumanDuration {
    pub fn parse(text: &str) -> Result<Self, String> {
        parse(text).map(Self)
    }
}

fn parse(text: &str) -> Result<Duration, String> {
    let trimmed = text.trim();
    let invalid = || format!("`{text}` is not a duration such as `750ms`, `30s`, `5m` or `1h`");

    // `ms` is tried before `s`, or every millisecond value would be read as seconds with a
    // stray `m` left on the number.
    let (value, millis_per_unit) = if let Some(head) = trimmed.strip_suffix("ms") {
        (head, 1)
    } else if let Some(head) = trimmed.strip_suffix('s') {
        (head, 1000)
    } else if let Some(head) = trimmed.strip_suffix('m') {
        (head, 60_000)
    } else if let Some(head) = trimmed.strip_suffix('h') {
        (head, 3_600_000)
    } else {
        (trimmed, 1000)
    };

    let amount: u64 = value.trim().parse().map_err(|_| invalid())?;

    amount
        .checked_mul(millis_per_unit)
        .map(Duration::from_millis)
        .ok_or_else(invalid)
}

impl<'de> Deserialize<'de> for HumanDuration {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;

        parse(&text).map(HumanDuration).map_err(de::Error::custom)
    }
}

impl Serialize for HumanDuration {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_each_unit() {
        assert_eq!(parse("750ms"), Ok(Duration::from_millis(750)));
        assert_eq!(parse("45s"), Ok(Duration::from_secs(45)));
        assert_eq!(parse("5m"), Ok(Duration::from_secs(300)));
        assert_eq!(parse("2h"), Ok(Duration::from_secs(7200)));
    }

    #[test]
    fn bare_number_is_seconds() {
        assert_eq!(parse("90"), Ok(Duration::from_secs(90)));
    }

    #[test]
    fn rejects_nonsense() {
        assert!(parse("soon").is_err());
        assert!(parse("5 weeks").is_err());
        assert!(parse("999999999999999999999ms").is_err());
    }

    #[test]
    fn round_trips_through_display() {
        for text in ["750ms", "45s", "5m", "2h"] {
            let parsed = HumanDuration(parse(text).unwrap());

            assert_eq!(parsed.to_string(), text);
        }
    }
}
