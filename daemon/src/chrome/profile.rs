use std::path::Path;

use tracing::warn;

/// Tells a Chromium profile that its last session ended cleanly.
///
/// A display is killed rather than closed: systemd stops the unit, the daemon restarts, the
/// machine loses power. Chromium records that as a crash and comes back asking whether to restore
/// its pages, in a dialog that sits over whatever the wall is supposed to be showing and that
/// nobody is standing there to answer. `--disable-session-crashed-bubble` does not cover every
/// build; the record in the profile is what every build reads.
///
/// Rewriting one field is deliberate. Deleting the file would take the window size, the zoom and
/// the permissions with it.
pub fn mark_clean_exit(profile: &Path) {
    let preferences = profile.join("Default").join("Preferences");

    let Ok(body) = std::fs::read_to_string(&preferences) else {
        return;
    };

    let Ok(mut document) = serde_json::from_str::<serde_json::Value>(&body) else {
        warn!(path = %preferences.display(), "chromium preferences are not json, leaving them alone");

        return;
    };

    let Some(profile_section) = document
        .get_mut("profile")
        .and_then(serde_json::Value::as_object_mut)
    else {
        return;
    };

    profile_section.insert("exit_type".to_string(), "Normal".into());
    profile_section.insert("exited_cleanly".to_string(), true.into());

    if let Err(error) = std::fs::write(&preferences, document.to_string()) {
        warn!(path = %preferences.display(), "cannot record a clean exit: {error}");
    }
}
