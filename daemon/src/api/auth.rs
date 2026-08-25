use poem_openapi::param::Header;

use crate::state::AppState;

use super::{ApiError, ApiResult};

pub type Authorization = Header<Option<String>>;

pub fn authorize(state: &AppState, presented: &Authorization) -> ApiResult<()> {
    let Some(expected) = state.admin_key.as_deref() else {
        return Ok(());
    };

    let presented = presented
        .0
        .as_deref()
        .and_then(|value| value.strip_prefix("Bearer "))
        .unwrap_or_default();

    if constant_time_eq(presented.as_bytes(), expected.as_bytes()) {
        Ok(())
    } else {
        Err(ApiError::unauthorized())
    }
}

/// Comparing byte by byte with an early return leaks the length of the matching prefix through
/// timing.
fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }

    left.iter()
        .zip(right)
        .fold(0_u8, |difference, (a, b)| difference | (a ^ b))
        == 0
}

#[cfg(test)]
mod tests {
    use super::constant_time_eq;

    #[test]
    fn keys_match_only_when_identical() {
        assert!(constant_time_eq(b"secret", b"secret"));
        assert!(!constant_time_eq(b"secret", b"secrez"));
    }

    #[test]
    fn a_length_difference_never_matches() {
        assert!(!constant_time_eq(b"secret", b"secretary"));
        assert!(!constant_time_eq(b"", b"secret"));
    }
}
