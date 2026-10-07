// Embeds what `mb new` and `mb serve` need, so the binary works anywhere:
// - the maimbrain-game skill's template/ and template-3d/ (under
//   .claude/skills in the monorepo, under skills/ in the public
//   maimbrain-kit repo)
// - the built web runtime (runtime/dist in the monorepo, runtime-dist/ in
//   the kit). Without it, `mb serve` says how to build it.
//
// The runtime is usually built *after* mb-cli has compiled once (`just
// runtime` itself runs `mb abi`), so the files are watched even while they
// don't exist yet: Cargo reruns a build script whose rerun-if-changed path is
// missing, so the next `cargo run -p mb-cli` after `just runtime` embeds it.
use std::path::PathBuf;

const RUNTIME_FILES: [&str; 5] = ["index.html", "runtime.js", "probe-worker.js", "mb_host_bg.wasm", "silence.wav"];

fn main() {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap()).join("../..").canonicalize().unwrap();
    // (skill dir, built runtime) for the monorepo, then for the kit.
    let layouts = [(".claude/skills/maimbrain-game", "runtime/dist"), ("skills/maimbrain-game", "runtime-dist")];
    let (skill, dist) = layouts
        .iter()
        .map(|(s, d)| (root.join(s), root.join(d)))
        .find(|(s, _)| s.join("template/manifest.toml").exists())
        .expect("can't find the maimbrain-game skill's template/");
    for t in ["template", "template-3d"] {
        let dir = skill.join(t).canonicalize().unwrap_or_else(|e| panic!("{}: {e}", skill.join(t).display()));
        println!("cargo:rerun-if-changed={}", dir.display());
        let var = if t == "template" { "MB_TEMPLATE_DIR" } else { "MB_TEMPLATE_3D_DIR" };
        println!("cargo:rustc-env={var}={}", dir.display());
    }

    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("runtime");
    std::fs::create_dir_all(&out).unwrap();
    let present = RUNTIME_FILES.iter().all(|f| dist.join(f).is_file());
    for f in RUNTIME_FILES {
        let src = dist.join(f);
        // Watched whether or not it exists yet (see the top of this file).
        println!("cargo:rerun-if-changed={}", src.display());
        let dest = out.join(f);
        if present {
            std::fs::copy(&src, &dest).unwrap_or_else(|e| panic!("{}: {e}", src.display()));
        } else {
            std::fs::write(&dest, b"").unwrap();
        }
    }
    println!("cargo:rustc-env=MB_RUNTIME_EMBEDDED={}", if present { "1" } else { "" });
    println!("cargo:rustc-env=MB_RUNTIME_DIST={}", dist.display());
}
