use std::path::Path;

/// `include_dir!` needs `../ui/dist` to exist at compile time, and a fresh
/// clone that has never run `trunk build` does not have it. Creating it here
/// keeps `cargo build` working on its own; the resulting binary serves a plain
/// error page instead of the UI, which the warning below is about.
fn main() {
    let dist = Path::new(env!("CARGO_MANIFEST_DIR")).join("../ui/dist");

    if let Err(err) = std::fs::create_dir_all(&dist) {
        println!("cargo:warning=Unable to create {}: {err}", dist.display());
    }

    if !dist.join("index.html").exists() {
        println!(
            "cargo:warning=../ui/dist/index.html is missing, so this binary will not serve the web UI. Run `trunk build` in ui/ and rebuild."
        );
    }

    println!("cargo:rerun-if-changed=../ui/dist");
}
