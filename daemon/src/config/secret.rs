use std::fmt;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SecretRef {
    Env { env: String },
    File { file: String },
    Inline(String),
}

impl SecretRef {
    pub fn resolve(&self) -> Result<String> {
        match self {
            Self::Env { env } => {
                std::env::var(env).map_err(|_| anyhow!("environment variable {env} is not set"))
            }
            Self::File { file } => std::fs::read_to_string(file)
                .map(|body| body.trim().to_string())
                .with_context(|| format!("cannot read secret from {file}")),
            Self::Inline(value) => Ok(value.clone()),
        }
    }

    /// An inline value never leaves the daemon.
    pub fn export(&self, placeholder: &str) -> Self {
        match self {
            Self::Inline(_) => Self::Env {
                env: placeholder.to_string(),
            },
            other => other.clone(),
        }
    }
}

impl fmt::Debug for SecretRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Env { env } => write!(f, "SecretRef::Env({env})"),
            Self::File { file } => write!(f, "SecretRef::File({file})"),
            Self::Inline(_) => write!(f, "SecretRef::Inline(<redacted>)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inline_never_appears_in_debug_output() {
        let secret = SecretRef::Inline("hunter2".to_string());

        assert!(!format!("{secret:?}").contains("hunter2"));
    }

    #[test]
    fn export_replaces_inline_with_a_usable_reference() {
        let exported = SecretRef::Inline("hunter2".to_string()).export("MISSIOND_ADMIN_KEY");

        assert!(matches!(exported, SecretRef::Env { env } if env == "MISSIOND_ADMIN_KEY"));
    }

    #[test]
    fn export_leaves_a_reference_alone() {
        let exported = SecretRef::File {
            file: "/run/secrets/key".to_string(),
        }
        .export("MISSIOND_ADMIN_KEY");

        assert!(matches!(exported, SecretRef::File { file } if file == "/run/secrets/key"));
    }

    #[test]
    fn missing_file_is_an_error_not_an_empty_credential() {
        let secret = SecretRef::File {
            file: "/nonexistent/missiond-test".to_string(),
        };

        assert!(secret.resolve().is_err());
    }
}
