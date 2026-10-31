use std::path::Path;

fn main() {
    let dist = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../web/dist");
    println!("cargo:rerun-if-changed={}", dist.display());
    if !dist.join("index.html").exists() {
        std::fs::create_dir_all(&dist).expect("create web/dist");
        std::fs::write(
            dist.join("index.html"),
            "<!doctype html><title>mail-sieve</title><p>The web UI was not built. Run <code>npm run build</code> in <code>web/</code>.</p>",
        )
        .expect("write placeholder index.html");
    }
}
