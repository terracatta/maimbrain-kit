//! End-to-end runs of `mb art`, `mb music`, `mb sfx`, `mb regen` and
//! `mb keys` with the offline mock provider. No network: keys come from a
//! throwaway file and no real provider is ever selected.

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("mb-gen-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn mb(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mb"))
        .args(args)
        .current_dir(dir)
        .env("MB_KEYS_FILE", dir.join("keys.json"))
        // Signed out: no credentials, so nothing asks a server about Maimbrain's keys.
        .env("MB_CONFIG_DIR", dir.join("config"))
        .env_remove("MB_KEYS_SOURCE")
        .env_remove("MB_PROVIDER")
        .env_remove("GEMINI_API_KEY")
        .env_remove("GOOGLE_API_KEY")
        .env_remove("GOOGLE_CLOUD_PROJECT")
        .env_remove("OPENAI_API_KEY")
        .env_remove("ELEVENLABS_API_KEY")
        .env_remove("XI_API_KEY")
        .env_remove("MB_MAX_COST")
        .env("MB_GEN_DATE", "2026-10-07T00:00:00Z")
        .output()
        .unwrap()
}

fn ok(o: &Output) -> String {
    let err = String::from_utf8_lossy(&o.stderr).to_string();
    assert!(o.status.success(), "mb failed:\n{err}");
    err
}

fn game(dir: &Path) -> PathBuf {
    let g = dir.join("g");
    std::fs::create_dir_all(g.join("assets")).unwrap();
    std::fs::write(g.join("manifest.toml"), "abi = 0\nid = \"com.example.g\"\nname = \"G\"\nversion = \"0.1.0\"\ncreator = \"@t\"\norientation = \"portrait\"\nlogical_size = [360, 640]\ninputs = [\"touch\"]\nperf_tier = \"lite\"\nstdlib = { mb2d = 1 }\n").unwrap();
    g
}

fn sidecar(p: &Path) -> serde_json::Value {
    let mut s = p.as_os_str().to_owned();
    s.push(".gen.json");
    serde_json::from_str(&std::fs::read_to_string(PathBuf::from(s)).unwrap()).unwrap()
}

#[test]
fn sprite_with_style_sidecar_and_reprocess() {
    let d = tmp("sprite");
    let g = game(&d);
    ok(&mb(&d, &["art", "init", "g"]));
    assert!(g.join("art/style.toml").exists() && g.join("art/.gitignore").exists());
    ok(&mb(&d, &["art", "sprite", "g", "hero", "a round moth", "--provider", "mock", "--res", "0.5K", "--seed", "7"]));
    let png = g.join("assets/hero.png");
    let bytes = std::fs::read(&png).unwrap();
    assert_eq!(&bytes[16..24], &[0, 0, 0, 128, 0, 0, 0, 128], "128×128 by default");
    let sc = sidecar(&png);
    assert_eq!(sc["provider"], "mock");
    assert_eq!(sc["seed"], 7);
    assert_eq!(sc["prompt"], "a round moth");
    assert_eq!(sc["date"], "2026-10-07T00:00:00Z");
    assert!(sc["full_prompt"].as_str().unwrap().contains("Use only these colors"), "style applied");
    assert_eq!(sc["style"]["path"], "art/style.toml");
    assert!(g.join(sc["raw"].as_str().unwrap()).exists());
    assert!(g.join("art/preview/hero.png").exists());
    // Reprocessing the raw output gives the same bytes, without a provider.
    ok(&mb(&d, &["regen", "g/assets/hero.png", "--reprocess"]));
    assert_eq!(std::fs::read(&png).unwrap(), bytes);
    // Same seed, same provider: same result.
    ok(&mb(&d, &["regen", "g/assets/hero.png"]));
    assert_eq!(std::fs::read(&png).unwrap(), bytes);
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn variants_pick_atlas_and_check() {
    let d = tmp("variants");
    let g = game(&d);
    let e = ok(&mb(&d, &["art", "sprite", "g", "enemy", "a spiky blob", "--provider", "mock", "--res", "0.5K", "--variants", "3", "--size", "64x64"]));
    assert!(e.contains("mb art pick"));
    assert!(g.join("art/variants/enemy-sheet.png").exists());
    assert!(!g.join("assets/enemy.png").exists());
    ok(&mb(&d, &["art", "pick", "g", "enemy", "2"]));
    assert!(g.join("assets/enemy.png").exists());
    ok(&mb(&d, &["art", "ui", "g", "button", "a round button", "--provider", "mock", "--res", "0.5K", "--size", "96x48"]));
    ok(&mb(&d, &["art", "atlas", "g", "sprites"]));
    let rs = std::fs::read_to_string(g.join("src/sprites.rs")).unwrap();
    assert!(rs.contains("pub const ENEMY: [f32; 4]") && rs.contains("pub const BUTTON: [f32; 4]"), "{rs}");
    let e = String::from_utf8_lossy(&mb(&d, &["art", "check", "g"]).stderr).to_string();
    assert!(e.contains("mock placeholder") && e.contains("mb art atlas of 2 images"), "{e}");
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn icon_background_frames_layers_tile() {
    let d = tmp("kinds");
    let g = game(&d);
    for args in [
        vec!["art", "icon", "g", "the hero's face"],
        vec!["art", "background", "g", "sky", "a dusk sky", "--size", "180x320"],
        vec!["art", "frames", "g", "walk", "walking", "--frames", "4", "--size", "48x48"],
        vec!["art", "layers", "g", "hills", "hills", "--count", "2", "--size", "200x100"],
        vec!["art", "tile", "g", "stone", "stones", "--grid", "8x8"],
    ] {
        let mut a = args.clone();
        a.extend(["--provider", "mock", "--res", "0.5K"]);
        ok(&mb(&d, &a));
    }
    let dims = |p: &str| {
        let b = std::fs::read(g.join(p)).unwrap();
        (u32::from_be_bytes(b[16..20].try_into().unwrap()), u32::from_be_bytes(b[20..24].try_into().unwrap()))
    };
    assert_eq!(dims("icon.png"), (256, 256));
    assert_eq!(dims("assets/sky.png"), (180, 320));
    assert_eq!(dims("assets/walk.png"), (192, 48));
    assert_eq!(dims("assets/hills_0.png"), (200, 100));
    assert_eq!(dims("assets/hills_1.png"), (200, 100));
    assert_eq!(dims("assets/stone.png"), (32, 32), "8×8 art pixels at the default pixel_scale 4");
    // The near layer used the far layer as a style reference.
    assert_eq!(sidecar(&g.join("assets/hills_1.png"))["references"][0]["path"], "assets/hills_0.png");
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn ip_refusal_cost_cap_and_dry_run_without_keys() {
    let d = tmp("guards");
    game(&d);
    let o = mb(&d, &["art", "sprite", "g", "x", "Pikachu in a hat", "--provider", "mock"]);
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("someone else's"));
    ok(&mb(&d, &["art", "sprite", "g", "x", "Pikachu in a hat", "--provider", "mock", "--allow-name", "pikachu", "--res", "0.5K"]));
    // No key: a dry run still shows the prompt and the price of the default model.
    let e = ok(&mb(&d, &["art", "sprite", "g", "y", "a lantern", "--dry-run"]));
    assert!(e.contains("gemini-nano-banana-2.1") && e.contains("$0.0336") && e.contains("prompt:"), "{e}");
    // The spending cap.
    let o = mb(&d, &["art", "sprite", "g", "y", "a lantern", "--dry-run", "--variants", "4", "--max-cost", "0.10"]);
    assert!(!o.status.success() && String::from_utf8_lossy(&o.stderr).contains("over the $0.10 limit"));
    // Without a dry run and without a key, a clear setup message.
    let o = mb(&d, &["art", "sprite", "g", "y", "a lantern"]);
    let e = String::from_utf8_lossy(&o.stderr);
    assert!(!o.status.success() && e.contains("GEMINI_API_KEY") && e.contains("ask the Maimbrain team"), "{e}");
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn keys_are_saved_outside_the_game_and_masked() {
    let d = tmp("keys");
    let mut c = Command::new(env!("CARGO_BIN_EXE_mb"))
        .args(["keys", "set", "elevenlabs"])
        .env("MB_KEYS_FILE", d.join("keys.json"))
        .env_remove("ELEVENLABS_API_KEY")
        .env_remove("XI_API_KEY")
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    use std::io::Write;
    c.stdin.take().unwrap().write_all(b"sk_test_1234567890abcdef\n").unwrap();
    assert!(c.wait().unwrap().success());
    let o = mb(&d, &["keys", "list"]);
    let e = String::from_utf8_lossy(&o.stderr);
    assert!(e.contains("sk_t…cdef") && !e.contains("1234567890"), "{e}");
    ok(&mb(&d, &["keys", "remove", "elevenlabs"]));
    let _ = std::fs::remove_dir_all(&d);
}

fn has_oggenc() -> bool {
    Command::new("oggenc").arg("--version").output().is_ok_and(|o| o.status.success())
}

#[test]
fn music_and_sfx() {
    let d = tmp("sound");
    let g = game(&d);
    let o = mb(&d, &["music", "g", "music", "bouncy marimba", "--provider", "mock", "--bars", "4", "--rate", "22050", "--seed", "3"]);
    if !has_oggenc() {
        // The processed audio is kept and the error says how to finish.
        let e = String::from_utf8_lossy(&o.stderr);
        assert!(!o.status.success() && e.contains("oggenc"), "{e}");
        assert!(g.join("art/raw/music.processed.wav").exists());
        let _ = std::fs::remove_dir_all(&d);
        return;
    }
    let e = ok(&o);
    assert!(e.contains("4 bars of 4/4") && e.contains("(clean)"), "{e}");
    let sc = sidecar(&g.join("assets/music.ogg"));
    assert_eq!(sc["result"]["bars"], 4);
    assert!(sc["result"]["seam_ratio"].as_f64().unwrap() < 4.0);
    assert!((sc["result"]["loud_rms_db"].as_f64().unwrap() + 20.0).abs() < 1.5);
    ok(&mb(&d, &["music", "g", "theme", "synth", "--provider", "mock", "--layers", "--rate", "22050"]));
    assert!(g.join("assets/theme_base.ogg").exists() && g.join("assets/theme_hi.ogg").exists());
    ok(&mb(&d, &["sfx", "g", "chime", "a glass chime", "--provider", "mock", "--seconds", "0.3"]));
    let sc = sidecar(&g.join("assets/chime.ogg"));
    assert!(sc["result"]["seconds"].as_f64().unwrap() < 0.45, "silence trimmed");
    ok(&mb(&d, &["regen", "g/assets/chime.ogg", "--reprocess"]));
    // No sfx key and no --provider: the procedural path is suggested.
    let o = mb(&d, &["sfx", "g", "boom", "an explosion"]);
    assert!(!o.status.success() && String::from_utf8_lossy(&o.stderr).contains("sfx.py"));
    let _ = std::fs::remove_dir_all(&d);
}
