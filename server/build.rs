use std::path::Path;

/// `include_dir!` needs `../ui/dist` to exist when the crate compiles, and a
/// fresh clone that has never run `trunk build` does not have it. Creating it
/// here is what keeps `cargo build` working on its own -- and it beats checking
/// a placeholder into the directory, because `trunk build` empties it.
///
/// The resulting binary compiles but serves a plain error page in place of the
/// interface, which is what the warning below is about.
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
