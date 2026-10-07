// Embeds what `mb new` and `mb serve` need, so the binary works anywhere:
// - the maimbrain-game skill's template/ (under .claude/skills in the
//   monorepo, under skills/ in the public maimbrain-kit repo)
// - the built web runtime (runtime/dist in the monorepo, runtime-dist/ in
//   the kit). Without it, `mb serve` explains how to get it.
use std::path::{Path, PathBuf};

const RUNTIME_FILES: [&str; 5] = ["index.html", "runtime.js", "probe-worker.js", "mb_host_bg.wasm", "silence.wav"];

fn main() {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../..");
    let template = [".claude/skills/maimbrain-game/template", "skills/maimbrain-game/template"]
        .iter()
        .map(|p| root.join(p))
        .find(|p| p.join("manifest.toml").exists())
        .expect("can't find the maimbrain-game skill's template/");
    let template = template.canonicalize().unwrap();
    println!("cargo:rerun-if-changed={}", template.display());
    println!("cargo:rustc-env=MB_TEMPLATE_DIR={}", template.display());

    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("runtime");
    std::fs::create_dir_all(&out).unwrap();
    let dist = ["runtime/dist", "runtime-dist"].iter().map(|p| root.join(p)).find(|p| p.join("runtime.js").exists());
    for f in RUNTIME_FILES {
        let dest = out.join(f);
        match &dist {
            Some(d) => {
                println!("cargo:rerun-if-changed={}", d.join(f).display());
                std::fs::copy(d.join(f), &dest).unwrap_or_else(|e| panic!("{}: {e}", d.join(f).display()));
            }
            None => std::fs::write(&dest, b"").unwrap(),
        }
    }
    println!("cargo:rustc-env=MB_RUNTIME_EMBEDDED={}", if dist.is_some() { "1" } else { "" });
    let _ = Path::new("");
}
