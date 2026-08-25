use std::path::Path;

// rust-embed requires the directory to exist at compile time. `just build` fills it
// from the web UI; a bare `cargo check` must still work on a fresh clone.
fn main() {
    let dist = Path::new(env!("CARGO_MANIFEST_DIR")).join("web-dist");
    std::fs::create_dir_all(&dist).expect("failed to create web-dist");
    println!("cargo:rerun-if-changed=web-dist");
}
