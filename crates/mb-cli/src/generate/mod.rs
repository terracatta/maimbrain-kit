//! Generated assets: `mb art` (images), `mb music`, `mb sfx`, `mb regen`,
//! `mb keys` and `mb models`. A provider makes the raw image or sound; mb
//! post-processes it to the platform's rules (sizes, transparency, pixel
//! grids, palettes, PNG size; loops, loudness, Ogg Vorbis), writes it into
//! the game's `assets/`, and records how it was made in a sidecar
//! (`<asset>.gen.json`) so it can be audited and regenerated.
//!
//! Layout inside a game directory:
//!     art/style.toml        the style guide (mb art init)
//!     art/raw/              what providers returned (git-ignored; lets `mb regen --reprocess` redo post-processing for free)
//!     art/variants/         candidates from --variants N, and a contact sheet to compare them
//!     art/preview/          each asset over a checkerboard, dark and light ground, enlarged: look at it
//!     assets/x.png + assets/x.png.gen.json

pub mod art;
pub mod atlas;
pub mod audio;
pub mod elevenlabs;
pub mod google;
pub mod img;
pub mod ip;
pub mod keys;
pub mod maimbrain;
pub mod mock;
pub mod openai;
pub mod provider;
pub mod sound;
pub mod style;

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub fn sha256_hex(b: &[u8]) -> String {
    use sha2::Digest;
    let d = sha2::Sha256::digest(b);
    d.iter().map(|x| format!("{x:02x}")).collect()
}

/// Now as an RFC 3339 UTC timestamp (MB_GEN_DATE overrides it, for reproducible tests).
pub fn now_utc() -> String {
    if let Ok(d) = std::env::var("MB_GEN_DATE") {
        return d;
    }
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) as i64;
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    // Civil date from days since 1970-01-01 (Howard Hinnant's algorithm).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z", rem / 3600, rem % 3600 / 60, rem % 60)
}

/// A fresh seed when none was given (recorded, so `mb regen --seed` can lock it).
pub fn fresh_seed() -> u64 {
    let t = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
    (mock::hash(&[&t.to_le_bytes(), &std::process::id().to_le_bytes()]) % 2_000_000_000) + 1
}

/// The provenance sidecar written next to every generated asset.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Sidecar {
    /// "mb <version> art sprite" etc.
    pub generator: String,
    pub date: String,
    pub provider: String,
    pub model: String,
    /// "maimbrain" when the call went through Maimbrain's keys (the server called the provider).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub via: Option<String>,
    /// The user's description of the asset.
    pub prompt: String,
    /// Exactly what was sent to the model.
    pub full_prompt: String,
    pub seed: u64,
    /// Whether the provider honors seeds (if not, regenerating gives a new result).
    pub seed_honored: bool,
    #[serde(default)]
    pub references: Vec<FileRef>,
    #[serde(default)]
    pub style: Option<FileRef>,
    pub est_cost_usd: Option<f64>,
    #[serde(default)]
    pub cost_usd: Option<f64>,
    /// The raw provider output, relative to the game directory.
    pub raw: String,
    pub raw_sha256: String,
    pub output_sha256: String,
    /// Names the IP check let through because of --allow-name.
    #[serde(default)]
    pub allowed_names: Vec<String>,
    /// Text the model returned alongside the asset, if any.
    #[serde(default)]
    pub model_text: Option<String>,
    /// What `mb regen` needs to do it again (the resolved options).
    pub recipe: Value,
    /// Facts about the result (size, colors, loop points, BPM…).
    #[serde(default)]
    pub result: Value,
    pub note: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct FileRef {
    pub path: String,
    pub sha256: String,
}

pub fn sidecar_path(asset: &Path) -> PathBuf {
    let mut s = asset.as_os_str().to_owned();
    s.push(".gen.json");
    PathBuf::from(s)
}

pub fn write_sidecar(asset: &Path, s: &Sidecar) -> Result<(), String> {
    let p = sidecar_path(asset);
    std::fs::write(&p, serde_json::to_string_pretty(s).unwrap() + "\n").map_err(|e| format!("{}: {e}", p.display()))
}

pub fn read_sidecar(asset: &Path) -> Result<Sidecar, String> {
    let p = sidecar_path(asset);
    let t = std::fs::read_to_string(&p).map_err(|_| format!("{} has no {} (it wasn't made by mb art/music/sfx)", asset.display(), p.display()))?;
    serde_json::from_str(&t).map_err(|e| format!("{}: {e}", p.display()))
}

pub fn note_for(provider: &str) -> String {
    match provider {
        "gemini" | "vertex" => "Generated with a Google model; Google embeds an imperceptible SynthID watermark.".into(),
        "openai" => "Generated with an OpenAI GPT Image model (C2PA metadata is not kept after post-processing).".into(),
        "elevenlabs" => "Generated with ElevenLabs; check your plan's license terms (the free plan has non-commercial terms).".into(),
        "mock" => "Placeholder from the offline mock provider: replace before publishing.".into(),
        _ => String::new(),
    }
}

/// Path relative to `base` with forward slashes (for sidecars and logs).
pub fn rel(path: &Path, base: &Path) -> String {
    let p = path.strip_prefix(base).unwrap_or(path);
    p.components().map(|c| c.as_os_str().to_string_lossy()).collect::<Vec<_>>().join("/")
}

/// The game directory an asset belongs to, from its sidecar's output path.
pub fn game_of(asset: &Path, out_rel: &str) -> PathBuf {
    let depth = out_rel.split('/').count();
    let mut g = asset.to_path_buf();
    for _ in 0..depth {
        g = g.parent().map(Path::to_path_buf).unwrap_or_default();
    }
    if g.as_os_str().is_empty() { PathBuf::from(".") } else { g }
}

/// The spending limit for one command: --max-cost, MB_MAX_COST, else $1.
pub fn max_cost(flag: Option<f64>) -> f64 {
    flag.or_else(|| std::env::var("MB_MAX_COST").ok().and_then(|v| v.parse().ok())).unwrap_or(1.0)
}

pub fn check_cost(est: Option<f64>, n: u32, limit: f64, what: &str) -> Result<(), String> {
    match est {
        Some(e) => {
            let total = e * n as f64;
            eprintln!(
                "  cost: est. ${e:.4}{} for {what} (provider list price, Oct 2026; `mb models` for the table)",
                if n > 1 { format!(" × {n} = ${total:.4}") } else { String::new() }
            );
            if total > limit + 1e-9 {
                return Err(format!(
                    "estimated ${total:.2} is over the ${limit:.2} limit for one command; pass --max-cost {:.2} (or set MB_MAX_COST) to go ahead",
                    total
                ));
            }
        }
        None => eprintln!("  cost: unknown for {what}"),
    }
    Ok(())
}

/// Makes art/ with a .gitignore for raw outputs and previews (they're big and reproducible).
pub fn ensure_art_dir(game: &Path) -> Result<PathBuf, String> {
    let art = game.join("art");
    std::fs::create_dir_all(art.join("raw")).map_err(|e| format!("{}: {e}", art.display()))?;
    let gi = art.join(".gitignore");
    if !gi.exists() {
        std::fs::write(&gi, "# Provider outputs, variants and previews: large, and reproducible with `mb regen`.\nraw/\nvariants/\npreview/\n")
            .map_err(|e| format!("{}: {e}", gi.display()))?;
    }
    Ok(art)
}

/// The manifest's logical size and orientation, if the directory is a game.
pub fn game_screen(game: &Path) -> ([u32; 2], bool) {
    let text = std::fs::read_to_string(game.join("manifest.toml")).unwrap_or_default();
    match mb_format::Manifest::parse(&text) {
        Ok(m) => {
            let [w, h] = m.logical_size;
            ([w, h], h >= w)
        }
        Err(_) => ([360, 640], true),
    }
}

#[derive(clap::Args)]
pub struct RegenArgs {
    /// A generated asset (assets/x.png, assets/music.ogg, icon.png): reads its .gen.json.
    pub asset: PathBuf,
    /// Use this seed (lock one you liked, or try another).
    #[arg(long)]
    pub seed: Option<u64>,
    /// A new seed (the default reuses the recorded one).
    #[arg(long)]
    pub new_seed: bool,
    /// Replace the description.
    #[arg(long)]
    pub prompt: Option<String>,
    /// Redo only the post-processing from the saved raw output (no provider call, free).
    #[arg(long)]
    pub reprocess: bool,
    /// Another provider (gemini, vertex, openai, elevenlabs, mock), or maimbrain to use Maimbrain's keys.
    #[arg(long)]
    pub provider: Option<String>,
    #[arg(long)]
    pub model: Option<String>,
    /// Only use your own API keys, never Maimbrain's (also MB_KEYS_SOURCE=own).
    #[arg(long)]
    pub own_keys: bool,
    /// Make N candidates in art/variants/ instead (images; pick one with `mb art pick`).
    #[arg(long, default_value_t = 1)]
    pub variants: u32,
    #[arg(long)]
    pub max_cost: Option<f64>,
    /// Show the prompt and the cost, call nothing.
    #[arg(long)]
    pub dry_run: bool,
}

pub fn regen(mut a: RegenArgs) -> Result<(), String> {
    a.provider = maimbrain::take_flags(a.provider.as_deref(), a.own_keys)?;
    let sc = read_sidecar(&a.asset)?;
    // --provider maimbrain on an asset made with a provider Maimbrain's keys don't cover: their default for its kind.
    if maimbrain::mode() == maimbrain::Mode::Maimbrain && a.provider.is_none() && !maimbrain::SERVER_PROVIDERS.contains(&sc.provider.as_str()) {
        a.provider = Some(if sc.recipe["kind"] == "sfx" { "elevenlabs" } else { "gemini" }.to_string());
    }
    let kind = sc.recipe["kind"].as_str().unwrap_or("").to_string();
    match kind.as_str() {
        "music" | "sfx" => sound::regen(&a, &sc),
        _ => art::regen(&a, &sc),
    }
}

/// `mb models`: the catalog with prices and which keys are set.
pub fn models() -> Result<(), String> {
    use provider::{MODELS, Media, price_label};
    // Signed out, this asks nothing.
    use maimbrain::Server as _;
    let server = maimbrain::Live.status().ok().flatten().filter(|s| s.allowed);
    for (media, title) in [(Media::Image, "Images (mb art)"), (Media::Music, "Music (mb music)"), (Media::Sfx, "Sound effects (mb sfx)")] {
        eprintln!("{title}");
        for m in MODELS.iter().filter(|m| m.media == media) {
            let key = if m.provider == "mock" || keys::lookup(m.provider).is_some() {
                "✓"
            } else if server.as_ref().is_some_and(|s| s.supports(m.provider, Some(m.id))) {
                "M"
            } else {
                "·"
            };
            eprintln!(
                "  {key} {:10} {:28} {}{}{}",
                m.provider,
                m.id,
                price_label(m),
                if m.default { " · default" } else { "" },
                if m.seed { "" } else { " · no seed" }
            );
            eprintln!("    {:10} {}", "", m.note);
        }
        eprintln!();
    }
    eprintln!("✓ = your key is set (mb keys list); M = no key of yours, but Maimbrain's keys cover it for your account.");
    eprintln!("{}", maimbrain::describe(&maimbrain::Live).1);
    eprintln!("Prices: ai.google.dev/gemini-api/docs/pricing, cloud.google.com/vertex-ai/generative-ai/pricing,");
    eprintln!("developers.openai.com/api/docs/pricing, elevenlabs.io/pricing/api (as of October 2026; check before big batches).");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_and_paths() {
        let d = now_utc();
        assert_eq!(d.len(), 20, "{d}");
        assert!(d.ends_with('Z'));
        assert_eq!(game_of(Path::new("games/x/assets/m.png"), "assets/m.png"), PathBuf::from("games/x"));
        assert_eq!(game_of(Path::new("games/x/icon.png"), "icon.png"), PathBuf::from("games/x"));
        assert_eq!(game_of(Path::new("icon.png"), "icon.png"), PathBuf::from("."));
        assert_eq!(rel(Path::new("games/x/assets/m.png"), Path::new("games/x")), "assets/m.png");
        assert_eq!(sidecar_path(Path::new("a/b.png")), PathBuf::from("a/b.png.gen.json"));
    }

    #[test]
    fn cost_limit() {
        assert!(check_cost(Some(0.04), 3, 1.0, "x").is_ok());
        assert!(check_cost(Some(0.134), 10, 1.0, "x").is_err());
    }
}
