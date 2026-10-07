//! `mb`: make, build, validate and publish Maimbrain games.
//!
//!     mb new frogger                  # a new game crate from the skill's template (--3d for mb3d)
//!     mb build frogger                # cargo build → wasm-opt → pack → validate
//!     mb login                        # sign in to maimbrain.com (device code)
//!     mb publish frogger              # build, upload, wait for validation
//!     mb validate feed/dev.maimbrain.stack-0.1.0.mbx
//!     mb serve frogger                # preview in a browser; reload to rebuild
//!     mb doctor                       # check the toolchain

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use clap::{Parser, Subcommand};
use mb_format::bundle::{ASSET_EXTENSIONS, SOURCE_EXTENSIONS};
use mb_format::manifest::PerfTier;
use mb_format::{Manifest, Report};

mod remote;
mod scaffold;
mod serve;

#[derive(Parser)]
#[command(name = "mb", version, about = "Make, build, validate and publish Maimbrain games")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Create a new game crate from the maimbrain-game skill's template (id com.maimbrain.<you>.<name>).
    /// Works signed out: without --id the id and creator are placeholders until you sign in.
    New {
        /// Directory to create; its name becomes the crate name.
        dir: PathBuf,
        /// Start from the 3D template (mb3d scene with an mb2d HUD) instead of the 2D one.
        #[arg(long = "3d")]
        three_d: bool,
        /// Bundle id to use (reverse-DNS, e.g. dev.example.frogger) instead of your account's namespace.
        #[arg(long)]
        id: Option<String>,
        /// Creator handle to use (e.g. @ada) instead of your account's.
        #[arg(long)]
        creator: Option<String>,
    },
    /// Sign in to the Maimbrain server (opens a browser to confirm a code).
    Login {
        /// Server URL (default https://maimbrain.com, or MB_SERVER).
        #[arg(long)]
        server: Option<String>,
    },
    /// Show who you're signed in as.
    Whoami,
    /// Build a game (or take a .mbx) and upload it for review.
    Publish {
        /// Game directory or .mbx bundle.
        path: PathBuf,
        /// Where `mb build` writes the bundle.
        #[arg(long, default_value = "feed")]
        out: PathBuf,
    },
    /// Check that the toolchain for building games is installed.
    Doctor,
    /// Preview a game in a desktop browser (rebuilds on reload when files change).
    Serve {
        /// Game directory or .mbx bundle.
        path: PathBuf,
        #[arg(long, default_value_t = 8765)]
        port: u16,
        /// Where `mb build` writes the bundle.
        #[arg(long, default_value = "feed")]
        out: PathBuf,
    },
    /// Compile a game crate to wasm, optimize it, pack it into a .mbx and validate it.
    Build {
        /// Game directory (manifest.toml, Cargo.toml, src/, icon.png, assets/).
        dir: PathBuf,
        /// Where to write the bundle.
        #[arg(long, default_value = "feed")]
        out: PathBuf,
    },
    /// Pack a game directory and an already-built wasm into a .mbx.
    Pack {
        dir: PathBuf,
        #[arg(long)]
        wasm: PathBuf,
        #[arg(long, default_value = "feed")]
        out: PathBuf,
    },
    /// Print the host import table (SPEC §5) as JSON, for the runtime.
    Abi,
    /// Write feed.json (SPEC §8) and icons/ for the given bundles, in order.
    Feed {
        bundles: Vec<PathBuf>,
        /// Directory the feed is served from; bundle URLs are relative to it.
        #[arg(long, default_value = "feed")]
        out: PathBuf,
    },
    /// Apply loop metering to a wasm file (what the client does before running a game).
    Meter {
        input: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Lay out runtime + game for a browser or headless run: <out>/runtime/, <out>/game/ (with boot.json).
    Stage {
        bundle: PathBuf,
        #[arg(long, default_value = "build/stage")]
        out: PathBuf,
        /// Built runtime (runtime/dist).
        #[arg(long, default_value = "runtime/dist")]
        runtime: PathBuf,
        #[arg(long, default_value_t = 1)]
        seed: u64,
        /// UTC date for the daily seed (YYYY-MM-DD).
        #[arg(long, default_value = "2026-10-06")]
        date: String,
    },
    /// Check a .mbx against the spec.
    Validate {
        bundle: PathBuf,
        /// Print the report as JSON (includes the full manifest).
        #[arg(long)]
        json: bool,
        /// Also write the bundle's icon.png here, if the bundle is valid (the server uses this).
        #[arg(long)]
        icon: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    let result = match Cli::parse().cmd {
        Cmd::New { dir, three_d, id, creator } => {
            // Signed out (no saved credentials) this makes no network request.
            let who = if id.is_some() && creator.is_some() { None } else { remote::whoami().ok() };
            let pair = who.as_ref().and_then(|w| Some((w["username"].as_str()?, w["namespace"].as_str()?)));
            scaffold::new_game(&dir, &scaffold::Options { three_d, id: id.as_deref(), creator: creator.as_deref(), who: pair })
        }
        Cmd::Login { server } => remote::login(server.as_deref()),
        Cmd::Whoami => remote::whoami().map(|w| println!("@{} ({})", w["username"].as_str().unwrap_or("?"), remote::server(None))),
        Cmd::Publish { path, out } => {
            if path.extension().is_some_and(|e| e == "mbx") {
                remote::publish(&path)
            } else {
                build(&path).and_then(|wasm| pack(&path, &wasm, &out)).and_then(|bundle| remote::publish(&bundle))
            }
        }
        Cmd::Doctor => doctor(),
        Cmd::Serve { path, port, out } => serve::serve(
            &path,
            port,
            |dir| build(dir).and_then(|wasm| pack(dir, &wasm, &out)),
            |bundle, site, runtime| stage(bundle, site, runtime, 1, "2026-10-06"),
        ),
        Cmd::Build { dir, out } => build(&dir).and_then(|wasm| pack(&dir, &wasm, &out)).map(|_| ()),
        Cmd::Pack { dir, wasm, out } => pack(&dir, &wasm, &out).map(|_| ()),
        Cmd::Validate { bundle, json, icon } => validate(&bundle, json, icon.as_deref()),
        Cmd::Stage { bundle, out, runtime, seed, date } => stage(&bundle, &out, &runtime, seed, &date),
        Cmd::Meter { input, output } => std::fs::read(&input)
            .map_err(|e| format!("{}: {e}", input.display()))
            .and_then(|w| mb_format::meter::instrument(&w))
            .and_then(|w| std::fs::write(&output, w).map_err(|e| e.to_string())),
        Cmd::Feed { bundles, out } => feed(&bundles, &out),
        Cmd::Abi => {
            println!("{}", abi_json());
            Ok(())
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn read_manifest(dir: &Path) -> Result<Manifest, String> {
    let path = dir.join("manifest.toml");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    Manifest::parse(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// The game crate's package name, from its Cargo.toml.
fn package_name(dir: &Path) -> Result<String, String> {
    let path = dir.join("Cargo.toml");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let doc: toml::Table = text.parse().map_err(|e| format!("{}: {e}", path.display()))?;
    doc.get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(str::to_string)
        .ok_or_else(|| format!("{}: no [package] name", path.display()))
}

fn build(dir: &Path) -> Result<PathBuf, String> {
    let manifest = read_manifest(dir)?;
    let name = package_name(dir)?;
    let max_memory = manifest.perf_tier.max_memory_pages() * 65536;
    eprintln!("building {name} ({:?}, max memory {} MiB)", manifest.perf_tier, max_memory >> 20);
    let status = Command::new("cargo")
        // The game's own Cargo.toml: works for a standalone crate and for a workspace member.
        .args(["build", "--manifest-path"])
        .arg(dir.join("Cargo.toml"))
        .args(["-p", &name, "--target", "wasm32-unknown-unknown", "--release"])
        // SPEC §3: the declared memory maximum is set by perf tier.
        .env("CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS", format!("-C link-arg=--max-memory={max_memory}"))
        .status()
        .map_err(|e| format!("running cargo: {e}"))?;
    if !status.success() {
        return Err(format!("cargo build failed for {name}"));
    }
    let target = workspace_target_dir(dir)?;
    let raw = target.join("wasm32-unknown-unknown/release").join(format!("{}.wasm", name.replace('-', "_")));
    let opt = raw.with_extension("opt.wasm");
    let wasm_opt = Command::new("wasm-opt")
        .args(["-O3", "--strip-debug", "--strip-producers"])
        .args(mb_format_features())
        .arg(&raw)
        .arg("-o")
        .arg(&opt)
        .status();
    match wasm_opt {
        Ok(s) if s.success() => Ok(opt),
        Ok(_) => Err("wasm-opt failed".into()),
        Err(_) => {
            eprintln!("warning: wasm-opt not found; packing unoptimized wasm");
            Ok(raw)
        }
    }
}

/// wasm-opt flags matching the features mb-format allows (Wasm 2.0).
fn mb_format_features() -> [&'static str; 7] {
    [
        "--enable-bulk-memory",
        "--enable-reference-types",
        "--enable-sign-ext",
        "--enable-nontrapping-float-to-int",
        "--enable-multivalue",
        "--enable-simd",
        "--enable-mutable-globals",
    ]
}

fn workspace_target_dir(dir: &Path) -> Result<PathBuf, String> {
    let out = Command::new("cargo")
        .args(["metadata", "--format-version", "1", "--no-deps", "--manifest-path"])
        .arg(dir.join("Cargo.toml"))
        .output()
        .map_err(|e| format!("running cargo metadata: {e}"))?;
    let meta: serde_json::Value = serde_json::from_slice(&out.stdout).map_err(|e| e.to_string())?;
    meta["target_directory"].as_str().map(PathBuf::from).ok_or_else(|| "cargo metadata: no target_directory".into())
}

fn pack(dir: &Path, wasm: &Path, out: &Path) -> Result<PathBuf, String> {
    let manifest = read_manifest(dir)?;
    let read = |p: &Path| std::fs::read(p).map_err(|e| format!("{}: {e}", p.display()));
    let mut files = BTreeMap::new();
    files.insert("manifest.toml".to_string(), read(&dir.join("manifest.toml"))?);
    files.insert("icon.png".to_string(), read(&dir.join("icon.png"))?);
    files.insert("game.wasm".to_string(), read(wasm)?);
    // SPEC §1: src/ holds the crate the wasm was built from.
    files.insert("src/Cargo.toml".to_string(), read(&dir.join("Cargo.toml"))?);
    add_tree(&dir.join("src"), "src/src", SOURCE_EXTENSIONS, &mut files)?;
    if dir.join("assets").is_dir() {
        add_tree(&dir.join("assets"), "assets", ASSET_EXTENSIONS, &mut files)?;
    }
    let bytes = mb_format::pack(&files).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(out).map_err(|e| format!("{}: {e}", out.display()))?;
    let path = out.join(format!("{}-{}.mbx", manifest.id, manifest.version));
    std::fs::write(&path, &bytes).map_err(|e| format!("{}: {e}", path.display()))?;
    let report = mb_format::validate(&bytes);
    print_report(&path, &report);
    if report.ok() { Ok(path) } else { Err(format!("{} failed validation", path.display())) }
}

/// What `mb build` and the maimbrain-game skill's scripts need.
fn doctor() -> Result<(), String> {
    let run = |cmd: &str, args: &[&str]| Command::new(cmd).args(args).output().ok().filter(|o| o.status.success());
    let mut ok = true;
    let mut check = |name: &str, found: bool, required: bool, fix: &str| {
        let mark = if found { "✓" } else if required { "✗" } else { "!" };
        eprintln!("{mark} {name}{}", if found { String::new() } else { format!("  →  {fix}") });
        ok &= found || !required;
    };
    check("cargo / rustc", run("cargo", &["--version"]).is_some(), true, "install Rust: https://rustup.rs");
    let targets = run("rustup", &["target", "list", "--installed"]).map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default();
    check("wasm32-unknown-unknown target", targets.contains("wasm32-unknown-unknown"), true, "rustup target add wasm32-unknown-unknown");
    check("wasm-opt (smaller, faster games)", run("wasm-opt", &["--version"]).is_some(), false, "brew install binaryen (optional)");
    check("python3 (the skill's art and sound scripts)", run("python3", &["--version"]).is_some(), false, "install Python 3");
    check("oggenc (the skill's sound script)", run("oggenc", &["--version"]).is_some(), false, "brew install vorbis-tools");
    match remote::whoami() {
        Ok(w) => check(&format!("signed in as @{}", w["username"].as_str().unwrap_or("?")), true, false, ""),
        Err(_) => check("signed in", false, false, "mb login"),
    }
    if ok { Ok(()) } else { Err("install the missing required tools above".into()) }
}

fn add_tree(root: &Path, prefix: &str, exts: &[&str], files: &mut BTreeMap<String, Vec<u8>>) -> Result<(), String> {
    let mut entries: Vec<_> = std::fs::read_dir(root)
        .map_err(|e| format!("{}: {e}", root.display()))?
        .filter_map(Result::ok)
        .collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') {
            continue;
        }
        let key = format!("{prefix}/{name}");
        if path.is_dir() {
            add_tree(&path, &key, exts, files)?;
        } else if path.extension().and_then(|e| e.to_str()).is_some_and(|e| exts.contains(&e.to_ascii_lowercase().as_str())) {
            files.insert(key, std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?);
        } else {
            eprintln!("warning: skipping {} (type not allowed in {prefix}/)", path.display());
        }
    }
    Ok(())
}

fn validate(path: &Path, json: bool, icon: Option<&Path>) -> Result<(), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let report = mb_format::validate(&bytes);
    if let (true, Some(out)) = (report.ok(), icon) {
        let bundle = mb_format::Bundle::open(&bytes).map_err(|r| r.errors.join("; "))?;
        let png = bundle.file("icon.png").ok_or("bundle has no icon.png")?;
        std::fs::write(out, png).map_err(|e| format!("{}: {e}", out.display()))?;
    }
    if json {
        println!(
            "{}",
            serde_json::json!({
                "ok": report.ok(),
                "errors": report.errors,
                "warnings": report.warnings,
                "sha256": report.sha256,
                "size": report.compressed_size,
                "wasm_size": report.wasm_size,
                "startup_size": report.startup_size,
                "id": report.manifest.as_ref().map(|m| m.id.clone()),
                "version": report.manifest.as_ref().map(|m| m.version.clone()),
                "manifest": report.manifest,
            })
        );
    } else {
        print_report(path, &report);
    }
    if report.ok() { Ok(()) } else { Err("validation failed".into()) }
}

fn stage(bundle: &Path, out: &Path, runtime: &Path, seed: u64, date: &str) -> Result<(), String> {
    let bytes = std::fs::read(bundle).map_err(|e| format!("{}: {e}", bundle.display()))?;
    let b = mb_format::Bundle::open(&bytes).map_err(|r| format!("{} is invalid: {}", bundle.display(), r.errors.join("; ")))?;
    let game = out.join("game");
    let _ = std::fs::remove_dir_all(out);
    for path in b.paths() {
        let dest = game.join(path);
        std::fs::create_dir_all(dest.parent().unwrap()).map_err(|e| e.to_string())?;
        // Serve the metered wasm, exactly as the iOS client does.
        let data = if path == "game.wasm" { b.runtime_wasm() } else { b.file(path).unwrap() };
        std::fs::write(&dest, data).map_err(|e| e.to_string())?;
    }
    let boot = serde_json::json!({
        "manifest": b.manifest,
        "seed": seed.to_string(),
        "dailySeed": mb_format::daily_seed(&b.manifest.id, date).to_string(),
        "store": {},
        "playerId": "00112233445566778899aabbccddeeff",
        "locale": "en-US",
        "insets": { "top": 0, "right": 0, "bottom": 0, "left": 0 },
        "stripHeight": 24,
    });
    std::fs::write(game.join("boot.json"), serde_json::to_string_pretty(&boot).unwrap()).map_err(|e| e.to_string())?;
    let rt = out.join("runtime");
    std::fs::create_dir_all(&rt).map_err(|e| e.to_string())?;
    for entry in std::fs::read_dir(runtime).map_err(|e| format!("{}: {e} (run `just runtime`)", runtime.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        std::fs::copy(entry.path(), rt.join(entry.file_name())).map_err(|e| e.to_string())?;
    }
    eprintln!("staged {} at {} — serve it and open /runtime/index.html", b.manifest.id, out.display());
    Ok(())
}

fn feed(bundles: &[PathBuf], out: &Path) -> Result<(), String> {
    let icons = out.join("icons");
    std::fs::create_dir_all(&icons).map_err(|e| e.to_string())?;
    let mut items = Vec::new();
    for path in bundles {
        let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let b = mb_format::Bundle::open(&bytes).map_err(|r| format!("{} is invalid: {}", path.display(), r.errors.join("; ")))?;
        let m = &b.manifest;
        std::fs::write(icons.join(format!("{}.png", m.id)), b.file("icon.png").unwrap()).map_err(|e| e.to_string())?;
        let name = path.file_name().unwrap().to_string_lossy();
        items.push(serde_json::json!({
            "id": m.id,
            "version": m.version,
            "url": name,
            "sha256": b.sha256,
            "size": bytes.len(),
            "title": m.name,
            "creator": m.creator,
            "icon": format!("icons/{}.png", m.id),
            "orientation": m.orientation,
            "inputs": m.inputs,
            "needs": m.needs,
            "perf_tier": m.perf_tier,
            "likes": 0,
            "remix_of": if m.remix.parent.is_empty() { serde_json::Value::Null } else { m.remix.parent.clone().into() },
        }));
    }
    let doc = serde_json::json!({ "version": 0, "items": items });
    let dest = out.join("feed.json");
    std::fs::write(&dest, serde_json::to_string_pretty(&doc).unwrap()).map_err(|e| e.to_string())?;
    eprintln!("wrote {} with {} items", dest.display(), bundles.len());
    Ok(())
}

fn abi_json() -> String {
    use mb_format::abi::{IMPORTS, Requires, Ty};
    let ty = |t: &Ty| match t {
        Ty::I32 => "i32",
        Ty::I64 => "i64",
        Ty::F32 => "f32",
        Ty::F64 => "f64",
    };
    let imports: Vec<_> = IMPORTS
        .iter()
        .map(|i| {
            let requires = match i.requires {
                Requires::Always => serde_json::Value::Null,
                Requires::Stdlib(s) => serde_json::json!({ "stdlib": s }),
                Requires::Sensor(s) => serde_json::json!({ "sensor": s.as_str() }),
                Requires::Capability(c) => serde_json::json!({ "capability": c.as_str() }),
            };
            serde_json::json!({
                "name": i.name,
                "params": i.params.iter().map(ty).collect::<Vec<_>>(),
                "results": i.results.iter().map(ty).collect::<Vec<_>>(),
                "requires": requires,
            })
        })
        .collect();
    serde_json::to_string_pretty(&serde_json::json!({ "abi": mb_format::manifest::ABI, "imports": imports })).unwrap()
}

fn print_report(path: &Path, r: &Report) {
    let status = if r.ok() { "ok" } else { "INVALID" };
    eprintln!("{status}: {}", path.display());
    if let Some(m) = &r.manifest {
        let tier = match m.perf_tier {
            PerfTier::Lite => "lite",
            PerfTier::Full => "full",
        };
        eprintln!("  {} {} by {} ({tier})", m.id, m.version, m.creator);
    }
    eprintln!(
        "  {} bytes · wasm {} bytes · startup set {} bytes · sha256 {}",
        r.compressed_size, r.wasm_size, r.startup_size, r.sha256
    );
    for w in &r.warnings {
        eprintln!("  warning: {w}");
    }
    for e in &r.errors {
        eprintln!("  error: {e}");
    }
}
