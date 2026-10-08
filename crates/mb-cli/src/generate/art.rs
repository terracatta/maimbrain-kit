//! `mb art`: generate images for a game and post-process them to the
//! platform's rules.
//!
//!     mb art init games/moth                         # art/style.toml
//!     mb art sprite games/moth hero "a moth with stained-glass wings"
//!     mb art frames games/moth flap "the moth flapping" --frames 4
//!     mb art background games/moth sky "a night sky over a cathedral"
//!     mb art layers games/moth hills "rolling hills at dusk" --count 3
//!     mb art tile games/moth glass "leaded glass panes"
//!     mb art ui games/moth button "a round glass button"
//!     mb art icon games/moth "the moth's face, glowing"
//!     mb art sprite games/moth cat "our cat as the hero" --ref ~/cat.jpg
//!     mb art atlas games/moth sprites assets/hero.png assets/flap.png
//!     mb art check games/moth
//!
//! Every kind: the style guide is applied, the prompt is checked for
//! someone else's IP, the cost is shown (and capped), the raw output is kept
//! in art/raw/, the result goes to assets/ with a provenance sidecar, and a
//! preview goes to art/preview/ to look at.

use std::path::{Path, PathBuf};

use clap::{Args, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::img::{self, Anchor, Image};
use super::provider::{self, ImageRequest, Media, MockHint, RefImage};
use super::style::{self, PromptParts, Style};
use super::{FileRef, Sidecar};

#[derive(Subcommand)]
pub enum ArtCmd {
    /// Write a starting style guide, art/style.toml (palette, style words, references).
    Init {
        game: PathBuf,
        /// Overwrite an existing one.
        #[arg(long)]
        force: bool,
    },
    /// A sprite with a transparent background (characters, enemies, items).
    Sprite(Gen),
    /// Animation frames packed into one sheet of equal cells.
    Frames {
        #[command(flatten)]
        gen_: Gen,
        /// Number of frames.
        #[arg(long, default_value_t = 4)]
        frames: u32,
        /// Columns in the sheet (default: all frames in one row, up to 8).
        #[arg(long)]
        cols: Option<u32>,
    },
    /// A full-screen, opaque background (default: the game's logical size × 2).
    Background(Gen),
    /// Parallax layers, far to near (<name>_0.png …): the far one opaque, the rest transparent, all wrapping horizontally.
    Layers {
        #[command(flatten)]
        gen_: Gen,
        #[arg(long, default_value_t = 3)]
        count: u32,
        /// Layer width in screens (they scroll and wrap).
        #[arg(long, default_value_t = 2.0)]
        screens: f32,
    },
    /// A seamless, tileable square texture.
    Tile(Gen),
    /// A UI element (button, panel, badge) with a transparent background.
    Ui(Gen),
    /// The game's icon.png (256×256, opaque): a close-up of your hero or hook, no text.
    Icon {
        game: PathBuf,
        prompt: String,
        #[command(flatten)]
        opts: Opts,
    },
    /// Use one of the candidates made with --variants N.
    Pick { game: PathBuf, name: String, variant: u32 },
    /// Pack PNGs into one atlas PNG plus a Rust table of source rects.
    Atlas {
        game: PathBuf,
        /// Atlas name: writes assets/<name>.png and src/<name>.rs.
        name: String,
        /// PNG files to pack (default: every generated sprite and ui PNG in assets/).
        inputs: Vec<PathBuf>,
        /// Clear texels between sprites (stops linear filtering bleeding).
        #[arg(long, default_value_t = 2)]
        padding: u32,
        /// Trim transparent borders (adds <NAME>_OFFSET and <NAME>_SIZE constants).
        #[arg(long)]
        trim: bool,
        /// Where to write the Rust table (default src/<name>.rs).
        #[arg(long)]
        rust: Option<PathBuf>,
        /// Delete the packed source PNGs and their sidecars afterwards.
        #[arg(long)]
        remove_inputs: bool,
    },
    /// Check a game's assets against the budgets and list what was generated, and how.
    Check { game: PathBuf },
}

#[derive(Args, Clone)]
pub struct Gen {
    /// Game directory.
    pub game: PathBuf,
    /// Asset name: writes assets/<name>.png.
    pub name: String,
    /// What it is: describe it plainly, as something original.
    pub prompt: String,
    #[command(flatten)]
    pub opts: Opts,
}

#[derive(Args, Clone, Default)]
pub struct Opts {
    /// Output size in PNG pixels, e.g. 128x128 (the canvas has 2 pixels per logical unit).
    #[arg(long, value_parser = parse_wh)]
    pub size: Option<[u32; 2]>,
    /// Pixel art: the grid in art pixels, e.g. 32x32 (output = grid × the style's pixel_scale).
    #[arg(long, value_parser = parse_wh)]
    pub grid: Option<[u32; 2]>,
    /// A photo or sketch whose subject to redraw in the game's style (repeatable).
    #[arg(long = "ref")]
    pub refs: Vec<PathBuf>,
    /// Seed (recorded; reuse it to reproduce where the provider honors seeds).
    #[arg(long)]
    pub seed: Option<u64>,
    /// gemini (default), vertex, openai or mock; maimbrain uses Maimbrain's keys (if your account has access).
    #[arg(long)]
    pub provider: Option<String>,
    /// Model id (see `mb models`).
    #[arg(long)]
    pub model: Option<String>,
    /// Only use your own API keys, never Maimbrain's (also MB_KEYS_SOURCE=own).
    #[arg(long)]
    pub own_keys: bool,
    /// Generation resolution: 0.5K, 1K, 2K or 4K (cost goes up with it).
    #[arg(long)]
    pub res: Option<String>,
    /// OpenAI quality: low, medium (default) or high.
    #[arg(long)]
    pub quality: Option<String>,
    /// Make N candidates in art/variants/ plus a contact sheet; `mb art pick` one.
    #[arg(long, default_value_t = 1)]
    pub variants: u32,
    /// Where the subject sits: center, or bottom (feet on the bottom margin).
    #[arg(long, value_enum)]
    pub anchor: Option<Anchor>,
    /// Empty margin around a sprite, in output pixels (pixel art: art pixels).
    #[arg(long)]
    pub margin: Option<u32>,
    /// Quantize to at most N colors.
    #[arg(long)]
    pub colors: Option<u32>,
    /// Size budget for this PNG in KB: quantize until it fits.
    #[arg(long)]
    pub max_kb: Option<u32>,
    /// Output path relative to the game (default assets/<name>.png).
    #[arg(long)]
    pub out: Option<String>,
    /// Ignore art/style.toml.
    #[arg(long)]
    pub no_style: bool,
    /// Keep the generated background (no transparency).
    #[arg(long)]
    pub keep_background: bool,
    /// Background keying tolerance (Oklab distance; default 0.09; raise if a halo is left).
    #[arg(long)]
    pub tolerance: Option<f64>,
    /// Let a flagged name through (it's yours, or a false positive). Recorded in the sidecar.
    #[arg(long)]
    pub allow_name: Vec<String>,
    /// Refuse if the estimated cost of this command is above this many USD (default 1.00, or MB_MAX_COST).
    #[arg(long)]
    pub max_cost: Option<f64>,
    /// Print the prompt and the cost without calling the provider.
    #[arg(long)]
    pub dry_run: bool,
}

fn parse_wh(s: &str) -> Result<[u32; 2], String> {
    let (w, h) = s.split_once(['x', 'X', '×']).ok_or("expected WxH, e.g. 128x128")?;
    let w: u32 = w.trim().parse().map_err(|_| "bad width")?;
    let h: u32 = h.trim().parse().map_err(|_| "bad height")?;
    if w == 0 || h == 0 || w > 4096 || h > 4096 {
        return Err("each side must be 1–4096 (mb2d's image limit)".into());
    }
    Ok([w, h])
}

/// The resolved recipe for one image: stored in the sidecar, replayed by `mb regen`.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Job {
    pub kind: String,
    pub name: String,
    pub subject: String,
    pub out: String,
    pub size: [u32; 2],
    #[serde(default)]
    pub grid: Option<[u32; 2]>,
    /// The grid came from --grid (kept on reprocess); otherwise it follows the style's pixel_scale.
    #[serde(default)]
    pub grid_explicit: bool,
    #[serde(default)]
    pub refs: Vec<String>,
    pub seed: u64,
    pub provider: String,
    pub model: String,
    pub resolution: String,
    #[serde(default)]
    pub quality: String,
    pub aspect: String,
    #[serde(default)]
    pub anchor: Anchor,
    #[serde(default)]
    pub margin: u32,
    #[serde(default)]
    pub colors: Option<u32>,
    #[serde(default)]
    pub max_kb: Option<u32>,
    #[serde(default)]
    pub frames: u32,
    #[serde(default)]
    pub cols: u32,
    /// (index, count) for parallax layers.
    #[serde(default)]
    pub layer: Option<(u32, u32)>,
    #[serde(default)]
    pub keyed: bool,
    #[serde(default = "tol")]
    pub tolerance: f64,
    #[serde(default)]
    pub no_style: bool,
    #[serde(default)]
    pub allow_names: Vec<String>,
}

fn tol() -> f64 {
    0.09
}

pub const ASPECTS: &[(&str, f32)] = &[
    ("1:1", 1.0),
    ("2:3", 0.6667),
    ("3:2", 1.5),
    ("3:4", 0.75),
    ("4:3", 1.3333),
    ("4:5", 0.8),
    ("5:4", 1.25),
    ("9:16", 0.5625),
    ("16:9", 1.7778),
    ("21:9", 2.3333),
];

/// The supported aspect ratio closest to w/h (in log space).
pub fn nearest_aspect(w: f32, h: f32) -> &'static str {
    let r = (w / h).ln();
    ASPECTS.iter().min_by(|a, b| (a.1.ln() - r).abs().partial_cmp(&(b.1.ln() - r).abs()).unwrap()).unwrap().0
}

pub fn run(cmd: ArtCmd) -> Result<(), String> {
    match cmd {
        ArtCmd::Init { game, force } => init(&game, force),
        ArtCmd::Sprite(g) => generate_kind("sprite", &g.game, &g.name, &g.prompt, &g.opts, Extra::default()),
        ArtCmd::Frames { gen_: g, frames, cols } => {
            generate_kind("frames", &g.game, &g.name, &g.prompt, &g.opts, Extra { frames, cols: cols.unwrap_or(0), ..Extra::default() })
        }
        ArtCmd::Background(g) => generate_kind("background", &g.game, &g.name, &g.prompt, &g.opts, Extra::default()),
        ArtCmd::Layers { gen_: g, count, screens } => layers(&g, count, screens),
        ArtCmd::Tile(g) => generate_kind("tile", &g.game, &g.name, &g.prompt, &g.opts, Extra::default()),
        ArtCmd::Ui(g) => generate_kind("ui", &g.game, &g.name, &g.prompt, &g.opts, Extra::default()),
        ArtCmd::Icon { game, prompt, opts } => generate_kind("icon", &game, "icon", &prompt, &opts, Extra::default()),
        ArtCmd::Pick { game, name, variant } => pick(&game, &name, variant),
        ArtCmd::Atlas { game, name, inputs, padding, trim, rust, remove_inputs } => {
            super::atlas::run(&game, &name, &inputs, padding, trim, rust.as_deref(), remove_inputs)
        }
        ArtCmd::Check { game } => check(&game),
    }
}

fn init(game: &Path, force: bool) -> Result<(), String> {
    let art = super::ensure_art_dir(game)?;
    let path = art.join("style.toml");
    if path.exists() && !force {
        return Err(format!("{} exists (--force to overwrite)", path.display()));
    }
    let name = std::fs::read_to_string(game.join("manifest.toml"))
        .ok()
        .and_then(|t| mb_format::Manifest::parse(&t).ok())
        .map(|m| m.name)
        .unwrap_or_else(|| game.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default());
    std::fs::write(&path, style::TEMPLATE.replace("NAME", &name)).map_err(|e| format!("{}: {e}", path.display()))?;
    std::fs::create_dir_all(art.join("ref")).map_err(|e| e.to_string())?;
    eprintln!("wrote {} — set the style words and palette, then `mb art sprite {} hero \"...\"`", path.display(), game.display());
    eprintln!("put style reference images in {} and list them under references", art.join("ref").display());
    Ok(())
}

#[derive(Default, Clone)]
struct Extra {
    frames: u32,
    cols: u32,
    layer: Option<(u32, u32)>,
    screens: f32,
}

fn default_size(kind: &str, style: &Style, screen: [u32; 2], extra: &Extra) -> [u32; 2] {
    if let Some(s) = style.kind(kind).size {
        return s;
    }
    match kind {
        "sprite" | "frames" => [128, 128],
        "background" => [screen[0] * 2, screen[1] * 2],
        "layer" => [((screen[0] * 2) as f32 * extra.screens.max(1.0)).round() as u32, screen[1] * 2],
        "tile" => [64, 64],
        "ui" => [256, 128],
        "icon" => [256, 256],
        _ => [128, 128],
    }
}

fn keyed_kind(kind: &str, layer: Option<(u32, u32)>) -> bool {
    match kind {
        "sprite" | "frames" | "ui" => true,
        "layer" => layer.is_some_and(|(i, _)| i > 0),
        _ => false,
    }
}

fn build_job(kind: &str, game: &Path, name: &str, subject: &str, o: &Opts, style: &Style, extra: &Extra) -> Result<Job, String> {
    if name.is_empty() || name.contains(['/', '\\']) || name.starts_with('.') {
        return Err(format!("bad asset name {name:?}"));
    }
    let (screen, _) = super::game_screen(game);
    let scale = style.pixel_scale.max(1);
    let mut size = o.size.unwrap_or_else(|| default_size(kind, style, screen, extra));
    let mut grid = o.grid;
    if grid.is_none() && style.pixel_art && kind != "background" && kind != "icon" {
        grid = Some([(size[0] / scale).max(1), (size[1] / scale).max(1)]);
    }
    if let Some(g) = grid
        && o.size.is_none()
    {
        size = [g[0] * scale, g[1] * scale];
    }
    if kind == "icon" && size != [256, 256] {
        return Err("icon.png must be 256×256".into());
    }
    if let Some(g) = grid
        && (size[0] % g[0] != 0 || size[1] % g[1] != 0)
    {
        return Err(format!("--size {}x{} isn't a whole multiple of --grid {}x{}", size[0], size[1], g[0], g[1]));
    }
    if let Some(g) = grid
        && matches!(kind, "sprite" | "frames")
        && g[0].min(g[1]) < 24
    {
        eprintln!(
            "  note: {}×{} art pixels is very coarse; detail will be lost. For a character or detailed object use the default size ({}×{} art pixels), or --grid.",
            g[0], g[1], 128 / scale, 128 / scale
        );
    }
    let frames = extra.frames.max(1);
    let cols = if kind == "frames" { if extra.cols > 0 { extra.cols.min(frames) } else { frames.min(8) } } else { 0 };
    let rows = if kind == "frames" { frames.div_ceil(cols) } else { 1 };
    // The provider sees frames in a near-square grid (very wide images aren't supported).
    let (aw, ah) = if kind == "frames" {
        let (gc, gr) = style::frame_grid(frames);
        (gc * size[0], gr * size[1])
    } else {
        (size[0], size[1])
    };
    let flag = super::maimbrain::take_flags(o.provider.as_deref(), o.own_keys)?;
    let provider = match provider::choose(flag.as_deref(), style.provider.as_deref(), &["gemini", "vertex", "openai"], "images") {
        Ok(p) => p,
        // A dry run shows what the default provider would get, key or not.
        Err(_) if o.dry_run => "gemini".to_string(),
        Err(e) => return Err(e),
    };
    let model = match o.model.clone() {
        Some(m) => m,
        None => match (&style.model, style.provider.as_deref()) {
            (Some(m), Some(p)) if p == provider => m.clone(),
            (Some(m), None) if provider::find_model(&provider, m).is_some() => m.clone(),
            _ => provider::default_model(&provider, Media::Image).map(|m| m.id.to_string()).ok_or(format!("{provider} has no image model"))?,
        },
    };
    let big = size[0].max(size[1]).max(if kind == "frames" { aw.max(ah) / 2 } else { 0 });
    let resolution = o.res.clone().or(style.resolution.clone()).unwrap_or_else(|| if big > 1100 || kind == "layer" { "2K".into() } else { "1K".into() });
    if !["0.5K", "1K", "2K", "4K"].contains(&resolution.as_str()) {
        return Err(format!("--res {resolution}: use 0.5K, 1K, 2K or 4K (uppercase K)"));
    }
    let refs = o.refs.iter().map(|r| ref_path(r, game)).collect();
    Ok(Job {
        kind: kind.into(),
        name: name.into(),
        subject: subject.into(),
        out: o.out.clone().unwrap_or_else(|| if kind == "icon" { "icon.png".into() } else { format!("assets/{name}.png") }),
        size,
        grid,
        grid_explicit: o.grid.is_some(),
        refs,
        seed: o.seed.unwrap_or_else(super::fresh_seed),
        provider,
        model,
        resolution,
        quality: o.quality.clone().unwrap_or_else(|| "medium".into()),
        aspect: nearest_aspect(aw as f32, ah as f32).into(),
        anchor: o.anchor.unwrap_or(if kind == "frames" { Anchor::Bottom } else { Anchor::Center }),
        margin: o.margin.unwrap_or(if grid.is_some() {
            1
        } else if kind == "ui" {
            2
        } else {
            (size[0].min(size[1]) / 16).max(1)
        }),
        colors: o.colors,
        max_kb: o.max_kb,
        frames: if kind == "frames" { frames } else { 0 },
        cols,
        layer: extra.layer,
        keyed: keyed_kind(kind, extra.layer) && !o.keep_background,
        tolerance: o.tolerance.unwrap_or(0.09),
        no_style: o.no_style,
        allow_names: o.allow_name.clone(),
    })
    .map(|mut j: Job| {
        if kind == "frames" && rows > 1 {
            j.cols = cols;
        }
        j
    })
}

/// A reference path as stored: relative to the game when inside it.
fn ref_path(p: &Path, game: &Path) -> String {
    let abs = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let (ap, ag) = (abs(p), abs(game));
    match ap.strip_prefix(&ag) {
        Ok(r) => super::rel(r, Path::new("")),
        Err(_) => ap.to_string_lossy().to_string(),
    }
}

fn read_ref(game: &Path, r: &str) -> Result<(RefImage, FileRef), String> {
    let p = if Path::new(r).is_absolute() { PathBuf::from(r) } else { game.join(r) };
    let bytes = std::fs::read(&p).map_err(|e| format!("reference {}: {e}", p.display()))?;
    if bytes.len() > 7 << 20 {
        return Err(format!("reference {} is {} MB; shrink it below 7 MB", p.display(), bytes.len() >> 20));
    }
    let mime = if bytes.starts_with(b"\x89PNG") {
        "image/png"
    } else if bytes.starts_with(&[0xff, 0xd8]) {
        "image/jpeg"
    } else if bytes.len() > 12 && &bytes[8..12] == b"WEBP" {
        "image/webp"
    } else {
        return Err(format!("reference {} isn't a PNG, JPEG or WebP (convert HEIC photos to JPEG first)", p.display()));
    };
    let fr = FileRef { path: r.to_string(), sha256: super::sha256_hex(&bytes) };
    Ok((RefImage { bytes, mime: mime.into() }, fr))
}

fn load_style(game: &Path, no_style: bool) -> Result<(Style, Option<FileRef>), String> {
    if no_style {
        return Ok((Style::default(), None));
    }
    let (s, sha) = Style::load(game)?;
    Ok((s, sha.map(|sha| FileRef { path: "art/style.toml".into(), sha256: sha })))
}

fn generate_kind(kind: &str, game: &Path, name: &str, prompt: &str, o: &Opts, extra: Extra) -> Result<(), String> {
    if !game.is_dir() {
        return Err(format!("{} isn't a directory (pass the game directory first)", game.display()));
    }
    let (style, _) = load_style(game, o.no_style)?;
    let job = build_job(kind, game, name, prompt, o, &style, &extra)?;
    execute(game, &job, o.variants.max(1), o.dry_run, super::max_cost(o.max_cost)).map(|_| ())
}

fn layers(g: &Gen, count: u32, screens: f32) -> Result<(), String> {
    if !(1..=6).contains(&count) {
        return Err("--count must be 1–6".into());
    }
    let (style, _) = load_style(&g.game, g.opts.no_style)?;
    let mut jobs = Vec::new();
    for i in 0..count {
        let depth = if count == 1 {
            "the only layer, filling the whole image".to_string()
        } else if i == 0 {
            "the farthest layer: the sky and the most distant shapes, filling the whole image, low contrast".to_string()
        } else if i == count - 1 {
            "the nearest layer: foreground shapes along the bottom of the image, darkest and most detailed".to_string()
        } else {
            format!("a middle-distance layer ({} of {count} from the back): shapes in the lower part of the image", i + 1)
        };
        let mut o = g.opts.clone();
        o.out = None;
        if i > 0 {
            // Later layers match the far layer's palette and light.
            let far = g.game.join(format!("assets/{}_0.png", g.name));
            if far.exists() || g.opts.dry_run {
                o.refs.insert(0, far);
            }
        }
        let extra = Extra { layer: Some((i, count)), screens, ..Extra::default() };
        let job = build_job("layer", &g.game, &format!("{}_{i}", g.name), &format!("{} — {depth}", g.prompt), &o, &style, &extra)?;
        jobs.push(job);
    }
    let est = estimate_job(&jobs[0]);
    super::check_cost(est.map(|e| e * count as f64), g.opts.variants.max(1), super::max_cost(g.opts.max_cost), &format!("{count} layers"))?;
    for mut job in jobs {
        // The far layer now exists for the ones after it.
        if job.layer.is_some_and(|(i, _)| i > 0) && !job.refs.iter().any(|r| r.ends_with("_0.png")) {
            let far = format!("assets/{}_0.png", g.name);
            if g.game.join(&far).exists() {
                job.refs.insert(0, far);
            }
        }
        execute(&g.game, &job, g.opts.variants.max(1), g.opts.dry_run, f64::INFINITY)?;
    }
    eprintln!("draw layer 0 first and scroll layer i at a speed that grows with i (e.g. 0.2, 0.5, 1.0); each wraps horizontally");
    Ok(())
}

fn estimate_job(job: &Job) -> Option<f64> {
    let q = ["low", "medium", "high"].iter().position(|q| *q == job.quality).unwrap_or(1);
    provider::find_model(&job.provider, &job.model).and_then(|m| provider::estimate(m, &job.resolution, q, 0.0))
}

/// Generates (or reprocesses) one job; returns the written asset path.
pub fn execute(game: &Path, job: &Job, variants: u32, dry_run: bool, limit: f64) -> Result<PathBuf, String> {
    let (style, style_ref) = load_style(game, job.no_style)?;
    let palette = style.palette_rgb()?;
    // Originality guard over everything that goes into the prompt.
    let verdict = super::ip::check(&[&job.subject, &style.style, &style.keywords.join(" "), &style.kind(&job.kind).prompt], &job.allow_names);
    if !verdict.ok() {
        return Err(super::ip::refusal_message(&verdict));
    }
    for w in &verdict.warnings {
        eprintln!("  warning: the prompt references a named style ({w}); make sure the result is your own look");
    }
    let key = if job.keyed { Some(style.key_color()?) } else { None };
    let mut refs = Vec::new();
    let mut ref_files = Vec::new();
    let style_refs: Vec<String> = if job.no_style { Vec::new() } else { style.references.iter().take(3).cloned().collect() };
    for r in style_refs.iter().chain(&job.refs) {
        if dry_run && !(if Path::new(r).is_absolute() { PathBuf::from(r) } else { game.join(r) }).exists() {
            continue;
        }
        let (ri, fr) = read_ref(game, r)?;
        refs.push(ri);
        ref_files.push(fr);
    }
    let (_, portrait) = super::game_screen(game);
    let full_prompt = style::compose(
        &style,
        &PromptParts {
            kind: &job.kind,
            subject: &job.subject,
            frames: job.frames,
            grid: job.grid.map(|g| (g[0], g[1])),
            // OpenAI makes real transparency (background: "transparent"); others get a flat color to key out.
            keyed: if job.provider == "openai" { None } else { key },
            portrait: job.size[1] >= job.size[0] || (job.kind == "background" && portrait),
            style_refs: style_refs.len().min(refs.len()),
            subject_refs: refs.len().saturating_sub(style_refs.len()),
        },
    );
    let model = provider::find_model(&job.provider, &job.model);
    eprintln!(
        "{} {} → {} ({}×{}{})",
        job.kind,
        job.name,
        job.out,
        job.size[0],
        job.size[1],
        job.grid.map(|g| format!(", {}×{} art pixels", g[0], g[1])).unwrap_or_default()
    );
    eprintln!(
        "  {} {} · {} · aspect {} · seed {}{}",
        job.provider,
        job.model,
        job.resolution,
        job.aspect,
        job.seed,
        if model.is_some_and(|m| !m.seed) { " (not honored)" } else { "" }
    );
    if model.is_none() && job.provider != "mock" {
        eprintln!("  note: {} isn't in mb's catalog (`mb models`); sending it anyway, cost unknown", job.model);
    }
    let est = estimate_job(job);
    super::check_cost(est, variants, limit, "this image")?;
    if dry_run {
        eprintln!("  prompt: {full_prompt}");
        if !ref_files.is_empty() {
            eprintln!("  references: {}", ref_files.iter().map(|r| r.path.as_str()).collect::<Vec<_>>().join(", "));
        }
        eprintln!("  (dry run: nothing generated)");
        return Ok(game.join(&job.out));
    }
    let prov = provider::pick(&job.provider, &job.model, est.map(|e| e * variants as f64))?;
    if prov.via().is_some() && !verdict.allowed.is_empty() {
        return Err(super::maimbrain::allow_name_refused(&verdict.allowed, &job.provider));
    }
    let art = super::ensure_art_dir(game)?;
    let mut outs = Vec::new();
    let mut previews = Vec::new();
    for v in 0..variants {
        let seed = job.seed + v as u64;
        let req = ImageRequest {
            model: job.model.clone(),
            prompt: full_prompt.clone(),
            refs: refs.clone(),
            aspect: job.aspect.clone(),
            resolution: job.resolution.clone(),
            transparent: job.keyed,
            seed: Some(seed),
            quality: ["low", "medium", "high"].iter().position(|q| *q == job.quality).unwrap_or(1),
            hint: MockHint { kind: job.kind.clone(), frames: job.frames, key, palette: palette.clone(), pixel_art: job.grid.is_some() },
        };
        eprint!("  generating with {}{}…", prov.name(), if variants > 1 { format!(" {}/{variants}", v + 1) } else { String::new() });
        let t0 = std::time::Instant::now();
        let out = prov.image(&req)?;
        eprintln!(" {:.1} s", t0.elapsed().as_secs_f32());
        if let Some(n) = prov.note() {
            eprintln!("  {n}");
        }
        let ext = if out.bytes.starts_with(&[0xff, 0xd8]) { "jpg" } else { "png" };
        let raw_rel = format!("art/raw/{}-{seed}.{ext}", job.name);
        std::fs::write(game.join(&raw_rel), &out.bytes).map_err(|e| format!("{raw_rel}: {e}"))?;
        let raw_img = Image::decode(&out.bytes).map_err(|e| format!("{}: {e}", job.provider))?;
        let mut j = job.clone();
        j.seed = seed;
        let (im, facts) = process(&j, &style, &palette, &raw_img)?;
        let png = encode_budget(&im, &j)?;
        let dest = if variants == 1 { game.join(&j.out) } else { art.join("variants").join(format!("{}-{}.png", j.name, v + 1)) };
        if let Some(d) = dest.parent() {
            std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
        }
        std::fs::write(&dest, &png).map_err(|e| format!("{}: {e}", dest.display()))?;
        let sc = Sidecar {
            generator: format!("mb {} art {}", env!("CARGO_PKG_VERSION"), j.kind),
            date: super::now_utc(),
            provider: j.provider.clone(),
            model: j.model.clone(),
            via: prov.via().map(str::to_string),
            prompt: j.subject.clone(),
            full_prompt: full_prompt.clone(),
            seed,
            seed_honored: model.is_none_or(|m| m.seed),
            references: ref_files.clone(),
            style: style_ref.clone(),
            est_cost_usd: est,
            cost_usd: out.cost_usd,
            raw: raw_rel,
            raw_sha256: super::sha256_hex(&out.bytes),
            output_sha256: super::sha256_hex(&png),
            allowed_names: verdict.allowed.clone(),
            model_text: out.text.clone(),
            recipe: serde_json::to_value(&j).unwrap(),
            result: facts.clone(),
            note: super::note_for(&j.provider),
        };
        super::write_sidecar(&dest, &sc)?;
        let preview = im.preview();
        std::fs::create_dir_all(art.join("preview")).map_err(|e| e.to_string())?;
        let pv = if variants == 1 {
            art.join("preview").join(format!("{}.png", j.name))
        } else {
            art.join("variants").join(format!("{}-{}.preview.png", j.name, v + 1))
        };
        std::fs::write(&pv, preview.encode_png()).map_err(|e| e.to_string())?;
        report(&dest, &im, png.len(), &facts, out.cost_usd);
        outs.push(dest);
        previews.push(im);
    }
    if variants > 1 {
        let sheet = art.join("variants").join(format!("{}-sheet.png", job.name));
        std::fs::write(&sheet, img::contact_sheet(&previews).encode_png()).map_err(|e| e.to_string())?;
        eprintln!("  compare: {} (variants 1–{variants}, left to right, top to bottom)", sheet.display());
        eprintln!("  then: mb art pick {} {} <n>", game.display(), job.name);
        println!("{}", sheet.display());
    } else {
        eprintln!("  look at it: {}", art.join("preview").join(format!("{}.png", job.name)).display());
        println!("{}", outs[0].display());
    }
    Ok(outs.remove(0))
}

fn report(dest: &Path, im: &Image, bytes: usize, facts: &serde_json::Value, cost: Option<f64>) {
    let colors = im.distinct_colors(256).len();
    eprintln!(
        "  wrote {} ({}×{}, {} KB, {} colors{}){}",
        dest.display(),
        im.w,
        im.h,
        bytes.div_ceil(1024),
        if colors > 256 { "256+".to_string() } else { colors.to_string() },
        if im.is_opaque() { ", opaque" } else { ", transparent" },
        cost.filter(|c| *c > 0.0).map(|c| format!(" · actual cost ${c:.4}")).unwrap_or_default()
    );
    if let Some(w) = facts["warnings"].as_array() {
        for w in w {
            eprintln!("  warning: {}", w.as_str().unwrap_or(""));
        }
    }
    if let Some(r) = facts["rust"].as_str() {
        eprintln!("  {r}");
    }
}

/// Encodes, quantizing until the PNG fits job.max_kb (if set); warns on big files.
fn encode_budget(im: &Image, job: &Job) -> Result<Vec<u8>, String> {
    let mut png = im.encode_png();
    if let Some(kb) = job.max_kb {
        let limit = kb as usize * 1024;
        let mut n = 256;
        while png.len() > limit && n >= 8 {
            let mut q = im.clone();
            img::quantize(&mut q, n);
            png = q.encode_png();
            n /= 2;
        }
        if png.len() > limit {
            eprintln!("  warning: still {} KB after quantizing to 8 colors (budget {kb} KB); use a smaller --size", png.len() / 1024);
        }
    } else {
        let warn_kb = match job.kind.as_str() {
            "tile" | "ui" | "icon" => 200,
            "sprite" => 400,
            _ => 1200,
        };
        if png.len() > warn_kb * 1024 {
            eprintln!(
                "  warning: {} KB is a lot for one {} (bundle ≤ 10 MB, startup set ≤ 3 MB); try --colors 128 or --max-kb {}",
                png.len() / 1024,
                job.kind,
                warn_kb / 2
            );
        }
    }
    Ok(png)
}

/// Post-processing by kind: keying, trimming, fitting, pixel grids, frames,
/// seamless wrapping, palettes. Returns the image and facts for the sidecar.
pub fn process(job: &Job, style: &Style, palette: &[[u8; 3]], raw: &Image) -> Result<(Image, serde_json::Value), String> {
    let mut warnings: Vec<String> = Vec::new();
    let mut facts = json!({});
    let keyed_bg = std::cell::Cell::new(None);
    let [w, h] = job.size;
    let flat_bg = palette.first().copied().unwrap_or([0, 0, 0]);
    // Transparency: keep a provider's real alpha, else key out the flat background.
    let keyed = |raw: &Image, warnings: &mut Vec<String>| -> Image {
        if raw.has_transparent_border() {
            return raw.clone();
        }
        let (k, rep) = img::key_out(raw, job.tolerance as f32);
        keyed_bg.set(Some(rep.background));
        if rep.noise > job.tolerance as f32 * 0.7 {
            warnings.push(format!("the background wasn't one flat color (variation {:.2}); edges may be rough. Regenerate, or raise --tolerance", rep.noise));
        }
        if job.kind != "layer" && rep.border_bg < 0.6 {
            warnings.push(format!(
                "the subject touches the image edges ({:.0}% of the border isn't background); it may be cut off. Regenerate",
                100.0 * (1.0 - rep.border_bg)
            ));
        }
        if rep.cleared < 0.05 {
            warnings.push("almost nothing was keyed out: the subject may fill the frame or the background isn't flat".into());
        }
        if rep.cleared > 0.985 {
            warnings.push("almost everything was keyed out: the subject may be the background's color; set a different `key` in art/style.toml".into());
        }
        k
    };
    let mut out = match job.kind.as_str() {
        "sprite" | "ui" => {
            let src = if job.keyed { keyed(raw, &mut warnings).trimmed() } else { raw.clone() };
            match job.grid {
                Some(g) => fit_pixel(&src, g, job.margin, job.anchor).scale_nearest(w / g[0]),
                None if job.keyed => src.contain(w, h, job.margin, job.anchor),
                None => src.cover(w, h),
            }
        }
        "frames" => {
            let k = if job.keyed { keyed(raw, &mut warnings) } else { raw.clone() };
            let n = job.frames.max(1);
            let (gc, gr) = style::frame_grid(n);
            let parts = split_frames(&k, n, gc, gr, &mut warnings);
            let cols = job.cols.max(1);
            let rows = n.div_ceil(cols);
            let mut sheet = Image::new(cols * w, rows * h, [0; 4]);
            // One scale for every frame, so the character doesn't change size.
            let maxw = parts.iter().map(|p| p.w).max().unwrap_or(1) as f32;
            let maxh = parts.iter().map(|p| p.h).max().unwrap_or(1) as f32;
            for (i, p) in parts.iter().enumerate() {
                let cell = match job.grid {
                    Some(g) => {
                        let (cw, ch) = (g[0] * 8, g[1] * 8);
                        let m = job.margin * 8;
                        let k = ((cw.saturating_sub(2 * m)) as f32 / maxw).min((ch.saturating_sub(2 * m)) as f32 / maxh);
                        let placed = place(p, k, cw, ch, m, job.anchor);
                        let mut c = img::pixelate(&placed, g[0], g[1]);
                        if style.palette_lock {
                            img::lock_palette(&mut c, palette);
                        }
                        c.scale_nearest(w / g[0])
                    }
                    None => {
                        let k = ((w.saturating_sub(2 * job.margin)) as f32 / maxw).min((h.saturating_sub(2 * job.margin)) as f32 / maxh);
                        place(p, k, w, h, job.margin, job.anchor)
                    }
                };
                sheet.blit(&cell, (i as u32 % cols) * w, (i as u32 / cols) * h);
            }
            facts["frames"] = json!(n);
            facts["cell"] = json!([w, h]);
            facts["cols"] = json!(cols);
            facts["rust"] = json!(format!("frame i's source rect: [(i % {cols}) × {w}, (i / {cols}) × {h}, {w}, {h}] (e.g. {} frames at 8–12 fps)", n));
            sheet
        }
        "background" | "icon" => {
            let base = if raw.is_opaque() { raw.clone() } else { raw.flatten(flat_bg) };
            match job.grid {
                Some(g) => img::pixelate(&base.cover(g[0] * 8, g[1] * 8), g[0], g[1]).scale_nearest(w / g[0]),
                None => base.cover(w, h),
            }
        }
        "layer" => {
            let src = if job.keyed {
                keyed(raw, &mut warnings)
            } else if raw.is_opaque() {
                raw.clone()
            } else {
                raw.flatten(flat_bg)
            };
            let band = 0.08;
            let wide = ((w as f32) * (1.0 + band)).round() as u32;
            let s = img::seamless(&src.cover(wide, h), true, false, band / (1.0 + band));
            let s = if s.w != w { s.resize(w, h) } else { s };
            facts["wraps"] = json!("x");
            match job.grid {
                Some(g) => img::pixelate(&s.resize(g[0] * 8, g[1] * 8), g[0], g[1]).scale_nearest(w / g[0]),
                None => s,
            }
        }
        "tile" => {
            let band = 0.125;
            let work = if let Some(g) = job.grid { (g[0] * 8, g[1] * 8) } else { (w, h) };
            let src = if raw.is_opaque() { raw.clone() } else { raw.flatten(flat_bg) };
            let (ww, wh) = (((work.0 as f32) * (1.0 + band)).round() as u32, ((work.1 as f32) * (1.0 + band)).round() as u32);
            let s = img::seamless(&src.cover(ww, wh), true, true, band / (1.0 + band));
            let s = if (s.w, s.h) != work { s.resize(work.0, work.1) } else { s };
            facts["wraps"] = json!("xy");
            match job.grid {
                Some(g) => img::pixelate(&s, g[0], g[1]).scale_nearest(w / g[0]),
                None => s,
            }
        }
        k => return Err(format!("unknown kind {k}")),
    };
    if job.kind != "frames" && (out.w, out.h) != (w, h) {
        out = out.resize(w, h);
    }
    if style.palette_lock && !palette.is_empty() {
        img::lock_palette(&mut out, palette);
    }
    let colors = job.colors.unwrap_or(style.colors);
    if colors > 0 {
        img::quantize(&mut out, colors as usize);
    }
    if job.kind == "icon" && !out.is_opaque() {
        out = out.flatten(flat_bg);
    }
    if let Some(g) = job.grid {
        facts["pixel_scale"] = json!(w / g[0]);
    }
    if let Some(bg) = keyed_bg.get() {
        facts["keyed_background"] = json!(img::hex(bg));
    }
    facts["size"] = json!([out.w, out.h]);
    facts["warnings"] = json!(warnings);
    Ok((out, facts))
}

/// Scales trimmed content by k and places it in a cw×ch cell.
fn place(p: &Image, k: f32, cw: u32, ch: u32, margin: u32, anchor: Anchor) -> Image {
    let nw = ((p.w as f32 * k).round() as u32).clamp(1, cw);
    let nh = ((p.h as f32 * k).round() as u32).clamp(1, ch);
    let s = p.resize(nw, nh);
    let mut c = Image::new(cw, ch, [0; 4]);
    let x = (cw - nw) / 2;
    let y = match anchor {
        Anchor::Center => (ch - nh) / 2,
        Anchor::Bottom => ch - margin.min(ch - nh) - nh,
    };
    c.blit(&s, x, y);
    c
}

/// Fits content into a pixel grid: scaled into a working canvas 8 texels per
/// art pixel, then reduced to the grid.
fn fit_pixel(src: &Image, g: [u32; 2], margin: u32, anchor: Anchor) -> Image {
    let work = src.contain(g[0] * 8, g[1] * 8, margin * 8, anchor);
    img::pixelate(&work, g[0], g[1])
}

/// Splits a keyed sprite sheet into its frames: connected shapes (with small
/// bits joined to the nearest big one), in reading order; falls back to an
/// even grid when the count doesn't match.
fn split_frames(k: &Image, n: u32, gc: u32, gr: u32, warnings: &mut Vec<String>) -> Vec<Image> {
    let (w, h) = (k.w as usize, k.h as usize);
    let mut label = vec![u32::MAX; w * h];
    let mut comps: Vec<(usize, [usize; 4])> = Vec::new(); // area, bbox x0 y0 x1 y1
    let mut stack = Vec::new();
    for s in 0..w * h {
        if k.px[s][3] < 24 || label[s] != u32::MAX {
            continue;
        }
        let id = comps.len() as u32;
        let mut area = 0;
        let mut bb = [usize::MAX, usize::MAX, 0, 0];
        stack.push(s);
        label[s] = id;
        while let Some(i) = stack.pop() {
            area += 1;
            let (x, y) = (i % w, i / w);
            bb = [bb[0].min(x), bb[1].min(y), bb[2].max(x), bb[3].max(y)];
            for (dx, dy) in [(-1i64, 0i64), (1, 0), (0, -1), (0, 1)] {
                let (nx, ny) = (x as i64 + dx, y as i64 + dy);
                if nx < 0 || ny < 0 || nx >= w as i64 || ny >= h as i64 {
                    continue;
                }
                let j = ny as usize * w + nx as usize;
                if k.px[j][3] >= 24 && label[j] == u32::MAX {
                    label[j] = id;
                    stack.push(j);
                }
            }
        }
        comps.push((area, bb));
    }
    let largest = comps.iter().map(|c| c.0).max().unwrap_or(0);
    let mut big: Vec<usize> = (0..comps.len()).filter(|&i| comps[i].0 * 4 >= largest).collect();
    if big.len() == n as usize {
        // Owner of each small component: the big one with the nearest bbox centre.
        let center = |b: &[usize; 4]| ((b[0] + b[2]) as f32 / 2.0, (b[1] + b[3]) as f32 / 2.0);
        let mut owner: Vec<usize> = (0..comps.len()).collect();
        for (i, c) in comps.iter().enumerate() {
            if !big.contains(&i) {
                let (cx, cy) = center(&c.1);
                owner[i] = *big
                    .iter()
                    .min_by(|a, b| {
                        let (ax, ay) = center(&comps[**a].1);
                        let (bx, by) = center(&comps[**b].1);
                        ((ax - cx).powi(2) + (ay - cy).powi(2)).partial_cmp(&((bx - cx).powi(2) + (by - cy).powi(2))).unwrap()
                    })
                    .unwrap();
            }
        }
        // Reading order: rows by centre y (row height = sheet height / rows), then x.
        let row_h = h as f32 / gr.max(1) as f32;
        big.sort_by(|a, b| {
            let (ax, ay) = center(&comps[*a].1);
            let (bx, by) = center(&comps[*b].1);
            ((ay / row_h) as u32, ax as u32).cmp(&((by / row_h) as u32, bx as u32))
        });
        return big
            .iter()
            .map(|&b| {
                let members: Vec<usize> = (0..comps.len()).filter(|&i| owner[i] == b).collect();
                let mut bb = comps[b].1;
                for &m in &members {
                    let c = comps[m].1;
                    bb = [bb[0].min(c[0]), bb[1].min(c[1]), bb[2].max(c[2]), bb[3].max(c[3])];
                }
                let mut im = Image::new((bb[2] - bb[0] + 1) as u32, (bb[3] - bb[1] + 1) as u32, [0; 4]);
                for y in bb[1]..=bb[3] {
                    for x in bb[0]..=bb[2] {
                        let l = label[y * w + x];
                        if l != u32::MAX && owner[l as usize] == b {
                            im.set((x - bb[0]) as u32, (y - bb[1]) as u32, k.px[y * w + x]);
                        }
                    }
                }
                im
            })
            .collect();
    }
    warnings.push(format!("found {} separate shapes for {n} frames; split the sheet into an even {gc}×{gr} grid instead", big.len()));
    let (cw, ch) = (k.w / gc, k.h / gr);
    (0..n).map(|i| k.crop((i % gc) * cw, (i / gc) * ch, cw, ch).trimmed()).collect()
}

fn pick(game: &Path, name: &str, variant: u32) -> Result<(), String> {
    let src = game.join("art/variants").join(format!("{name}-{variant}.png"));
    let sc = super::read_sidecar(&src)?;
    let out = sc.recipe["out"].as_str().ok_or("variant sidecar has no output path")?.to_string();
    let dest = game.join(&out);
    if let Some(d) = dest.parent() {
        std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
    }
    std::fs::copy(&src, &dest).map_err(|e| format!("{}: {e}", dest.display()))?;
    super::write_sidecar(&dest, &sc)?;
    let pv = game.join("art/variants").join(format!("{name}-{variant}.preview.png"));
    if pv.exists() {
        let _ = std::fs::create_dir_all(game.join("art/preview"));
        let _ = std::fs::copy(&pv, game.join("art/preview").join(format!("{name}.png")));
    }
    eprintln!("picked variant {variant} (seed {}) → {}", sc.seed, dest.display());
    println!("{}", dest.display());
    Ok(())
}

pub fn regen(a: &super::RegenArgs, sc: &Sidecar) -> Result<(), String> {
    let mut job: Job = serde_json::from_value(sc.recipe.clone()).map_err(|e| format!("sidecar recipe: {e}"))?;
    let asset_rel = super::rel(&a.asset, Path::new(""));
    if !asset_rel.ends_with(&job.out) {
        return Err(format!("{} is a candidate; `mb art pick` it first, then regenerate {}", a.asset.display(), job.out));
    }
    let game = super::game_of(&a.asset, &job.out);
    if let Some(s) = a.seed {
        job.seed = s;
    } else if a.new_seed {
        job.seed = super::fresh_seed();
    }
    if let Some(p) = &a.prompt {
        job.subject = p.clone();
    }
    if let Some(p) = &a.provider {
        job.provider = p.clone();
        if a.model.is_none() {
            job.model = provider::default_model(p, Media::Image).map(|m| m.id.to_string()).unwrap_or_default();
        }
    }
    if let Some(m) = &a.model {
        job.model = m.clone();
    }
    if a.reprocess {
        let raw_path = game.join(&sc.raw);
        let raw =
            std::fs::read(&raw_path).map_err(|e| format!("{}: {e} (raw outputs live in art/raw/; regenerate without --reprocess)", raw_path.display()))?;
        let (style, style_ref) = load_style(&game, job.no_style)?;
        let palette = style.palette_rgb()?;
        // A grid --grid didn't fix follows the style as it is now (pixel_art, pixel_scale).
        if !job.grid_explicit {
            let scale = style.pixel_scale.max(1);
            job.grid = (style.pixel_art && job.kind != "background" && job.kind != "icon" && job.size[0] % scale == 0 && job.size[1] % scale == 0)
                .then(|| [(job.size[0] / scale).max(1), (job.size[1] / scale).max(1)]);
        }
        let (im, facts) = process(&job, &style, &palette, &Image::decode(&raw)?)?;
        let png = encode_budget(&im, &job)?;
        let dest = game.join(&job.out);
        std::fs::write(&dest, &png).map_err(|e| format!("{}: {e}", dest.display()))?;
        let mut sc2 = sc.clone();
        sc2.output_sha256 = super::sha256_hex(&png);
        sc2.style = style_ref;
        sc2.recipe = serde_json::to_value(&job).unwrap();
        sc2.result = facts.clone();
        super::write_sidecar(&dest, &sc2)?;
        let art = super::ensure_art_dir(&game)?;
        std::fs::create_dir_all(art.join("preview")).map_err(|e| e.to_string())?;
        std::fs::write(art.join("preview").join(format!("{}.png", job.name)), im.preview().encode_png()).map_err(|e| e.to_string())?;
        eprintln!("reprocessed {} from {} (no provider call)", job.out, sc.raw);
        report(&dest, &im, png.len(), &facts, None);
        println!("{}", dest.display());
        return Ok(());
    }
    execute(&game, &job, a.variants.max(1), a.dry_run, super::max_cost(a.max_cost)).map(|_| ())
}

/// `mb art check`: budgets and provenance for a game's assets.
fn check(game: &Path) -> Result<(), String> {
    let text = std::fs::read_to_string(game.join("manifest.toml")).map_err(|e| format!("{}: {e}", game.join("manifest.toml").display()))?;
    let manifest = mb_format::Manifest::parse(&text)?;
    let (_, style_sha) = Style::load(game)?;
    let mut files = Vec::new();
    collect(&game.join("assets"), &mut files);
    files.sort();
    let mut total = 0usize;
    let mut startup = 0usize;
    let mut spent = 0.0;
    let mut problems = 0;
    let mut generated = 0;
    let mut images = 0;
    eprintln!("{:34} {:>9} {:>11}  made by", "asset", "KB", "size");
    let icon = game.join("icon.png");
    for path in std::iter::once(icon.clone()).chain(files) {
        let name = super::rel(&path, game);
        if name.ends_with(".gen.json") || !path.is_file() {
            continue;
        }
        let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
        if path != icon {
            total += bytes.len();
            if manifest.startup_assets.contains(&name) {
                startup += bytes.len();
            }
        }
        let mut notes = Vec::new();
        let mut dims = String::new();
        if name.ends_with(".png") {
            images += (path != icon) as usize;
            if let Some((w, h)) = img::png_dims(&bytes) {
                dims = format!("{w}×{h}");
                if w > 4096 || h > 4096 {
                    notes.push("over mb2d's 4096² limit".to_string());
                    problems += 1;
                }
                if path == icon && (w, h) != (256, 256) {
                    notes.push("icon.png must be 256×256".into());
                    problems += 1;
                }
            }
            if bytes.len() > 1200 << 10 {
                notes.push("big: quantize (--colors) or shrink".into());
            }
        }
        if name.ends_with(".ogg")
            && let Ok((ch, rate)) = super::audio::decode(&bytes, "audio/ogg")
        {
            dims = format!("{:.1} s", ch[0].len() as f32 / rate as f32);
            if ch.len() > 1 {
                notes.push("stereo (SPEC: use mono)".into());
            }
        }
        let made = match super::read_sidecar(&path) {
            Ok(sc) => {
                generated += 1;
                spent += sc.cost_usd.or(sc.est_cost_usd).unwrap_or(0.0);
                if sc.provider == "mock" {
                    notes.push("mock placeholder: replace before publishing".into());
                }
                if let (Some(s), Some(cur)) = (&sc.style, &style_sha)
                    && &s.sha256 != cur
                {
                    notes.push("made with an older style guide (mb regen to refresh)".into());
                }
                if sc.output_sha256 != super::sha256_hex(&bytes) {
                    notes.push("edited since it was generated".into());
                }
                format!("{} {} seed {}", sc.provider, sc.model, sc.seed)
            }
            Err(_) => match std::fs::read_to_string(super::sidecar_path(&path)).ok().and_then(|t| serde_json::from_str::<serde_json::Value>(&t).ok()) {
                Some(v) if v["packed_from"].is_array() => format!("mb art atlas of {} images", v["packed_from"].as_array().unwrap().len()),
                _ => "by hand or a script".into(),
            },
        };
        eprintln!(
            "{:34} {:>9} {:>11}  {made}{}",
            name,
            bytes.len().div_ceil(1024),
            dims,
            if notes.is_empty() { String::new() } else { format!("  ← {}", notes.join("; ")) }
        );
    }
    eprintln!();
    eprintln!(
        "assets {:.2} MB of the 10 MB bundle (plus wasm); startup assets {:.2} MB of 3 MB (plus wasm); {images} images (≤ 128)",
        total as f64 / 1048576.0,
        startup as f64 / 1048576.0
    );
    eprintln!("{generated} generated asset(s), about ${spent:.2} spent on them");
    if total > 9 << 20 || startup > 3 << 20 || images > 128 {
        problems += 1;
        eprintln!("over budget: quantize big PNGs (mb regen <asset> --reprocess after setting colors in art/style.toml), lower music quality, or cut assets");
    }
    if problems > 0 { Err(format!("{problems} problem(s)")) } else { Ok(()) }
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(rd) = std::fs::read_dir(dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                collect(&p, out);
            } else {
                out.push(p);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aspects() {
        assert_eq!(nearest_aspect(128.0, 128.0), "1:1");
        assert_eq!(nearest_aspect(720.0, 1280.0), "9:16");
        assert_eq!(nearest_aspect(1440.0, 1280.0), "5:4");
        assert_eq!(nearest_aspect(1000.0, 100.0), "21:9");
        assert!(parse_wh("128x64").is_ok());
        assert!(parse_wh("5000x64").is_err());
    }

    fn job(kind: &str) -> Job {
        Job {
            kind: kind.into(),
            name: "t".into(),
            subject: "x".into(),
            out: "assets/t.png".into(),
            size: [64, 64],
            grid_explicit: false,
            grid: None,
            refs: vec![],
            seed: 1,
            provider: "mock".into(),
            model: "mock-image".into(),
            resolution: "0.5K".into(),
            quality: "medium".into(),
            aspect: "1:1".into(),
            anchor: Anchor::Center,
            margin: 2,
            colors: None,
            max_kb: None,
            frames: 0,
            cols: 0,
            layer: None,
            keyed: true,
            tolerance: 0.09,
            no_style: false,
            allow_names: vec![],
        }
    }

    fn mock_raw(j: &Job) -> Image {
        let req = ImageRequest {
            model: "mock-image".into(),
            prompt: "a creature".into(),
            refs: vec![],
            aspect: j.aspect.clone(),
            resolution: "0.5K".into(),
            transparent: j.keyed,
            seed: Some(3),
            quality: 1,
            hint: MockHint { kind: j.kind.clone(), frames: j.frames, key: Some([255, 0, 255]), palette: vec![], pixel_art: j.grid.is_some() },
        };
        super::super::mock::image(&req)
    }

    #[test]
    fn sprite_is_exact_transparent_and_clean() {
        let j = job("sprite");
        let (im, facts) = process(&j, &Style::default(), &[], &mock_raw(&j)).unwrap();
        assert_eq!((im.w, im.h), (64, 64));
        assert_eq!(im.get(0, 0)[3], 0);
        assert_eq!(im.get(32, 32)[3], 255);
        let (x, y, w, h) = im.content_bbox(0).unwrap();
        assert!(x >= 2 && y >= 2 && x + w <= 62 && y + h <= 62, "margin kept: {x} {y} {w} {h}");
        // No magenta fringe survives.
        assert!(!im.px.iter().any(|p| p[3] > 60 && p[0] > 200 && p[1] < 60 && p[2] > 200), "magenta left");
        assert!(facts["warnings"].as_array().unwrap().is_empty(), "{facts}");
    }

    #[test]
    fn pixel_sprite_has_whole_art_pixels_in_palette() {
        let mut j = job("sprite");
        j.grid = Some([16, 16]);
        j.size = [64, 64];
        j.margin = 1;
        let style = Style { palette: vec!["#1d1b2f".into(), "#f2c14e".into(), "#5d8aa8".into(), "#fdf6e3".into()], palette_lock: true, ..Style::default() };
        let pal = style.palette_rgb().unwrap();
        let (im, _) = process(&j, &style, &pal, &mock_raw(&j)).unwrap();
        assert_eq!((im.w, im.h), (64, 64));
        for y in 0..64 {
            for x in 0..64 {
                assert_eq!(im.get(x, y), im.get(x / 4 * 4, y / 4 * 4), "4×4 blocks");
            }
        }
        for p in im.px.iter().filter(|p| p[3] > 0) {
            assert!(pal.contains(&[p[0], p[1], p[2]]) && p[3] == 255);
        }
    }

    #[test]
    fn frames_sheet() {
        let mut j = job("frames");
        j.frames = 4;
        j.cols = 4;
        j.aspect = "1:1".into();
        j.anchor = Anchor::Bottom;
        let (im, facts) = process(&j, &Style::default(), &[], &mock_raw(&j)).unwrap();
        assert_eq!((im.w, im.h), (256, 64));
        assert!(facts["warnings"].as_array().unwrap().is_empty(), "{facts}");
        // Every cell has a character standing on the bottom margin.
        for i in 0..4 {
            let cell = im.crop(i * 64, 0, 64, 64);
            let (_, y, _, h) = cell.content_bbox(0).expect("frame drawn");
            assert_eq!(y + h, 62, "frame {i} bottom-anchored");
        }
    }

    #[test]
    fn tile_wraps_and_background_is_opaque() {
        let mut j = job("tile");
        j.keyed = false;
        let (t, _) = process(&j, &Style::default(), &[], &mock_raw(&j)).unwrap();
        assert_eq!((t.w, t.h), (64, 64));
        assert!(img::wrap_error(&t, true) < 0.05 && img::wrap_error(&t, false) < 0.05);
        let mut b = job("background");
        b.keyed = false;
        b.size = [90, 160];
        b.aspect = "9:16".into();
        let (bg, _) = process(&b, &Style::default(), &[], &mock_raw(&b)).unwrap();
        assert_eq!((bg.w, bg.h), (90, 160));
        assert!(bg.is_opaque());
    }

    #[test]
    fn budget_quantizes() {
        let mut j = job("background");
        j.keyed = false;
        j.size = [256, 256];
        j.max_kb = Some(12);
        let (bg, _) = process(&j, &Style::default(), &[], &mock_raw(&j)).unwrap();
        let png = encode_budget(&bg, &j).unwrap();
        assert!(png.len() <= 12 * 1024, "{}", png.len());
    }
}
