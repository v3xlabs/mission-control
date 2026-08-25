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
        let total = self.0.as_secs();

        if total > 0 && total.is_multiple_of(3600) {
            write!(f, "{}h", total / 3600)
        } else if total > 0 && total.is_multiple_of(60) {
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
    let (value, multiplier) = match trimmed.strip_suffix(['s', 'm', 'h']) {
        Some(head) => {
            let unit = trimmed.as_bytes()[trimmed.len() - 1];
            let multiplier = match unit {
                b's' => 1,
                b'm' => 60,
                _ => 3600,
            };

            (head, multiplier)
        }
        None => (trimmed, 1),
    };

    let amount: u64 = value
        .trim()
        .parse()
        .map_err(|_| format!("`{text}` is not a duration such as `30s`, `5m` or `1h`"))?;

    Ok(Duration::from_secs(amount * multiplier))
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
    }

    #[test]
    fn round_trips_through_display() {
        for text in ["45s", "5m", "2h"] {
            let parsed = HumanDuration(parse(text).unwrap());

            assert_eq!(parsed.to_string(), text);
        }
    }
}
