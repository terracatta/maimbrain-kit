//! `mb music` and `mb sfx`: generate sound and post-process it to the
//! platform's audio rules (SPEC §5.6, the skill's sfx.py and check_audio.py):
//! mono Ogg Vorbis, loudness-normalized (music ≈ −20 dBFS RMS, effects
//! ≈ −14), peaks ≤ −1 dBFS decoded; music cut to a whole number of bars with
//! a cross-faded seam so `play_looped` repeats it without a click.
//!
//!     mb music games/moth music "dreamy harp and glass chimes, gentle pulse" --bars 8
//!     mb music games/moth music "driving synthwave" --bpm 128 --layers   # music_base.ogg + music_hi.ogg
//!     mb music games/moth win "a short triumphant fanfare" --once
//!     mb sfx games/moth chime "a bright glass chime"
//!     mb sfx games/moth wind "soft wind" --loop --seconds 4

use std::path::{Path, PathBuf};

use clap::Args;
use serde::{Deserialize, Serialize};
use serde_json::json;

use super::audio::{self, Kind};
use super::provider::{self, AudioRequest, Media};
use super::style::Style;
use super::{FileRef, Sidecar};

#[derive(Args, Clone, Default)]
pub struct Common {
    /// Seed (recorded; honored by Lyria and the mock).
    #[arg(long)]
    pub seed: Option<u64>,
    /// gemini (music default), vertex, elevenlabs (sfx default) or mock; maimbrain uses Maimbrain's keys
    /// (if your account has access).
    #[arg(long)]
    pub provider: Option<String>,
    /// Model id (see `mb models`).
    #[arg(long)]
    pub model: Option<String>,
    /// Only use your own API keys, never Maimbrain's (also MB_KEYS_SOURCE=own).
    #[arg(long)]
    pub own_keys: bool,
    /// Output path relative to the game (default assets/<name>.ogg).
    #[arg(long)]
    pub out: Option<String>,
    /// Sample rate of the Ogg file (22050–48000).
    #[arg(long, default_value_t = 44100)]
    pub rate: u32,
    /// Vorbis quality (oggenc -q, 0–10; lower is smaller).
    #[arg(long, default_value_t = 4.0)]
    pub quality: f32,
    /// Ignore art/style.toml's [music]/[sfx] prompt.
    #[arg(long)]
    pub no_style: bool,
    /// Let a flagged name through (it's yours, or a false positive). Recorded in the sidecar.
    #[arg(long)]
    pub allow_name: Vec<String>,
    /// Refuse if the estimated cost is above this many USD (default 1.00, or MB_MAX_COST).
    #[arg(long)]
    pub max_cost: Option<f64>,
    /// Print the prompt and the cost without calling the provider.
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Args, Clone)]
pub struct MusicArgs {
    /// Game directory.
    pub game: PathBuf,
    /// Asset name: writes assets/<name>.ogg.
    pub name: String,
    /// Genre, instruments, mood, energy. Original: no "in the style of <artist>".
    pub prompt: String,
    /// Loop length in bars of 4/4 (default 8, fewer if it doesn't fit).
    #[arg(long)]
    pub bars: Option<u32>,
    /// Tempo: asked for in the prompt and used instead of detecting it.
    #[arg(long)]
    pub bpm: Option<f32>,
    /// Loop length in seconds, ignoring the beat (ambient pieces without a steady pulse).
    #[arg(long)]
    pub seconds: Option<f32>,
    /// Also split into <name>_base.ogg (low-passed, calm) and <name>_hi.ogg (the rest);
    /// start both together and raise the high layer's volume with intensity.
    #[arg(long)]
    pub layers: bool,
    /// Crossover frequency for --layers, Hz.
    #[arg(long, default_value_t = 700.0)]
    pub split_hz: f32,
    /// A one-shot (jingle, sting, game-over tune) instead of a loop.
    #[arg(long)]
    pub once: bool,
    /// What to keep out (Vertex lyria-002 only).
    #[arg(long)]
    pub negative: Option<String>,
    #[command(flatten)]
    pub common: Common,
}

#[derive(Args, Clone)]
pub struct SfxArgs {
    /// Game directory.
    pub game: PathBuf,
    /// Asset name: writes assets/<name>.ogg.
    pub name: String,
    /// What it sounds like: source, material, action, length, character.
    pub prompt: String,
    /// Length in seconds (0.5–30 for ElevenLabs; default: the model decides).
    #[arg(long)]
    pub seconds: Option<f32>,
    /// Make it loop seamlessly (engines, wind, ambience).
    #[arg(long = "loop")]
    pub looping: bool,
    #[command(flatten)]
    pub common: Common,
}

/// The resolved recipe for a sound, stored in the sidecar.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SoundJob {
    pub kind: String,
    pub name: String,
    pub subject: String,
    pub out: String,
    pub provider: String,
    pub model: String,
    pub seed: u64,
    #[serde(default)]
    pub bars: Option<u32>,
    #[serde(default)]
    pub bpm: Option<f32>,
    #[serde(default)]
    pub seconds: Option<f32>,
    #[serde(default)]
    pub layers: bool,
    #[serde(default = "split")]
    pub split_hz: f32,
    #[serde(default)]
    pub once: bool,
    #[serde(default)]
    pub looping: bool,
    #[serde(default)]
    pub negative: Option<String>,
    pub rate: u32,
    pub quality: f32,
    #[serde(default)]
    pub no_style: bool,
    #[serde(default)]
    pub allow_names: Vec<String>,
}

fn split() -> f32 {
    700.0
}

fn resolve(kind: &str, game: &Path, name: &str, subject: &str, c: &Common, style: &Style) -> Result<SoundJob, String> {
    if name.is_empty() || name.contains(['/', '\\']) || name.starts_with('.') {
        return Err(format!("bad asset name {name:?}"));
    }
    if !(22050..=48000).contains(&c.rate) {
        return Err("--rate must be 22050–48000 (SPEC §5.6)".into());
    }
    let _ = game;
    let media = if kind == "music" { Media::Music } else { Media::Sfx };
    let order: &[&str] = if kind == "music" { &["gemini", "vertex", "elevenlabs"] } else { &["elevenlabs"] };
    let style_provider = style.provider.as_deref().filter(|p| provider::default_model(p, media).is_some());
    let flag = super::maimbrain::take_flags(c.provider.as_deref(), c.own_keys)?;
    let chosen = provider::choose(flag.as_deref(), style_provider, order, if kind == "music" { "music" } else { "sound effects" });
    let chosen = if chosen.is_err() && c.dry_run { Ok(order[0].to_string()) } else { chosen };
    let provider = chosen.map_err(|e| {
        if kind == "sfx" {
            format!(
                "{e}\n\nNo key? Effects are quick to make procedurally and cost nothing: the maimbrain-game skill's scripts/sfx.py \
                 (8-bit voices, sweeps, noise, a step sequencer) writes Ogg files that already meet these rules."
            )
        } else {
            e
        }
    })?;
    let model = c
        .model
        .clone()
        .or_else(|| provider::default_model(&provider, media).map(|m| m.id.to_string()))
        .ok_or(format!("{provider} has no {kind} model (see `mb models`)"))?;
    Ok(SoundJob {
        kind: kind.into(),
        name: name.into(),
        subject: subject.into(),
        out: c.out.clone().unwrap_or_else(|| format!("assets/{name}.ogg")),
        provider,
        model,
        seed: c.seed.unwrap_or_else(super::fresh_seed),
        bars: None,
        bpm: None,
        seconds: None,
        layers: false,
        split_hz: 700.0,
        once: false,
        looping: false,
        negative: None,
        rate: c.rate,
        quality: c.quality,
        no_style: c.no_style,
        allow_names: c.allow_name.clone(),
    })
}

pub fn music(a: MusicArgs) -> Result<(), String> {
    let style = if a.common.no_style { Style::default() } else { Style::load(&a.game)?.0 };
    let mut job = resolve("music", &a.game, &a.name, &a.prompt, &a.common, &style)?;
    job.bars = a.bars;
    job.bpm = a.bpm;
    job.seconds = a.seconds;
    job.layers = a.layers;
    job.split_hz = a.split_hz;
    job.once = a.once;
    job.negative = a.negative;
    execute(&a.game, &job, a.common.dry_run, super::max_cost(a.common.max_cost), None)
}

pub fn sfx(a: SfxArgs) -> Result<(), String> {
    let style = if a.common.no_style { Style::default() } else { Style::load(&a.game)?.0 };
    let mut job = resolve("sfx", &a.game, &a.name, &a.prompt, &a.common, &style)?;
    job.seconds = a.seconds;
    job.looping = a.looping;
    execute(&a.game, &job, a.common.dry_run, super::max_cost(a.common.max_cost), None)
}

pub fn compose(job: &SoundJob, style: &Style) -> String {
    let mut p = vec![format!("{}.", job.subject.trim().trim_end_matches('.'))];
    let extra = style.kind(&job.kind).prompt;
    if !job.no_style && !extra.trim().is_empty() {
        p.push(format!("{}.", extra.trim().trim_end_matches('.')));
    }
    if job.kind == "music" {
        p.push("Instrumental only, no vocals.".into());
        if job.once {
            p.push("A short, complete piece with a clear ending.".into());
        } else {
            p.push(format!(
                "{}A steady tempo and one consistent groove all the way through: no intro, no breakdown, no fade-out and no ending, so it can loop seamlessly as game music.",
                job.bpm.map(|b| format!("{b:.0} BPM, 4/4. ")).unwrap_or_default()
            ));
        }
        if job.model == "lyria-3.5" {
            p.push(format!("About {:.0} seconds long.", job.seconds.unwrap_or(45.0).max(30.0)));
        }
        p.push("An original composition, not imitating any existing song or artist.".into());
    } else {
        p.push(format!(
            "A single clean sound effect for a mobile game{}, starting immediately with no silence before it; no music, no voice.",
            job.seconds.map(|s| format!(", about {s:.1} seconds")).unwrap_or_default()
        ));
        if job.looping {
            p.push("It loops seamlessly.".into());
        }
    }
    p.join(" ")
}

/// Generates (or with `raw`, reprocesses) a sound job.
pub fn execute(game: &Path, job: &SoundJob, dry_run: bool, limit: f64, raw: Option<(Vec<u8>, String, String)>) -> Result<(), String> {
    if !game.is_dir() {
        return Err(format!("{} isn't a directory (pass the game directory first)", game.display()));
    }
    let (style, style_sha) = if job.no_style { (Style::default(), None) } else { Style::load(game)? };
    let style_ref = style_sha.map(|s| FileRef { path: "art/style.toml".into(), sha256: s });
    let verdict = super::ip::check(&[&job.subject, &style.kind(&job.kind).prompt], &job.allow_names);
    if !verdict.ok() {
        return Err(super::ip::refusal_message(&verdict));
    }
    for w in &verdict.warnings {
        eprintln!("  warning: the prompt references a named style ({w}); make it your own sound");
    }
    let full_prompt = compose(job, &style);
    let model = provider::find_model(&job.provider, &job.model);
    let req_secs = match job.kind.as_str() {
        "music" if job.provider == "elevenlabs" || job.provider == "mock" => Some(job.seconds.unwrap_or(30.0).max(16.0) + 6.0),
        "music" => job.seconds,
        _ => job.seconds,
    };
    let est = model.and_then(|m| provider::estimate(m, "", 0, req_secs.unwrap_or(if job.kind == "sfx" { 2.0 } else { 30.0 })));
    eprintln!("{} {} → {}{}", job.kind, job.name, job.out, if job.layers { " (+ _base/_hi layers)" } else { "" });
    eprintln!("  {} {} · seed {}{}", job.provider, job.model, job.seed, if model.is_some_and(|m| !m.seed) { " (not honored)" } else { "" });
    let (bytes, mime, raw_rel, cost, text, via) = match raw {
        Some((b, m, r)) => {
            eprintln!("  reprocessing {r} (no provider call)");
            // Keep how the raw output was made.
            let via = super::read_sidecar(&game.join(&job.out)).ok().and_then(|s| s.via);
            (b, m, r, None, None, via)
        }
        None => {
            super::check_cost(est, 1, limit, &format!("this {}", if job.kind == "sfx" { "effect" } else { "track" }))?;
            if dry_run {
                eprintln!("  prompt: {full_prompt}\n  (dry run: nothing generated)");
                return Ok(());
            }
            let prov = provider::pick(&job.provider, &job.model, est)?;
            if prov.via().is_some() && !verdict.allowed.is_empty() {
                return Err(super::maimbrain::allow_name_refused(&verdict.allowed, &job.provider));
            }
            let req = AudioRequest {
                model: job.model.clone(),
                prompt: full_prompt.clone(),
                kind: if job.kind == "music" { Kind::Music } else { Kind::Sfx },
                seconds: req_secs,
                looping: job.looping,
                seed: Some(job.seed),
                negative: job.negative.clone(),
            };
            eprint!("  generating…");
            let t0 = std::time::Instant::now();
            let out = prov.audio(&req)?;
            eprintln!(" {:.1} s", t0.elapsed().as_secs_f32());
            if let Some(n) = prov.note() {
                eprintln!("  {n}");
            }
            super::ensure_art_dir(game)?;
            let ext = match out.mime.as_str() {
                "audio/mpeg" | "audio/mp3" => "mp3",
                "audio/ogg" => "ogg",
                m if m.starts_with("audio/pcm") => "pcm",
                _ => "wav",
            };
            let r = format!("art/raw/{}-{}.{ext}", job.name, job.seed);
            std::fs::write(game.join(&r), &out.bytes).map_err(|e| format!("{r}: {e}"))?;
            // Keep the mime for raw PCM next to it.
            if ext == "pcm" {
                let _ = std::fs::write(game.join(format!("{r}.mime")), &out.mime);
            }
            (out.bytes, out.mime, r, out.cost_usd, out.text, prov.via().map(str::to_string))
        }
    };
    let (chans, src_rate) = audio::decode(&bytes, &mime).map_err(|e| format!("{}: {e}", job.provider))?;
    let mono = audio::resample(&audio::to_mono(&chans), src_rate, job.rate);
    let rate = job.rate;
    let mut facts = json!({ "source_seconds": mono.len() as f32 / rate as f32, "source_channels": chans.len(), "source_rate": src_rate });
    let mut warnings: Vec<String> = Vec::new();
    // (file suffix, samples) to write.
    let outputs: Vec<(&str, Vec<f32>)> = if job.kind == "music" && !job.once {
        let lp = make_loop(&mono, rate, job, &mut facts, &mut warnings)?;
        if job.layers {
            let (base, hi) = audio::split_layers(&lp, rate, job.split_hz);
            let g = audio::gain_for(&lp, Kind::Music);
            vec![("_base", audio::limit(&base, g, rate)), ("_hi", audio::limit(&hi, g, rate))]
        } else {
            vec![("", audio::normalize(&lp, Kind::Music, rate))]
        }
    } else if job.kind == "music" {
        vec![("", audio::normalize(&audio::trim_silence(&mono, rate, -55.0), Kind::Music, rate))]
    } else if job.looping {
        let xf = ((0.05 * rate as f32) as usize).min(mono.len() / 4);
        let lp = audio::cut_loop(&mono, 0, mono.len() - xf, xf);
        vec![("", audio::normalize(&lp, Kind::Sfx, rate))]
    } else {
        let mut t = audio::trim_silence(&mono, rate, -50.0);
        // Hold the model to the length asked for (with a little slack).
        if let Some(s) = job.seconds
            && t.len() as f32 > s * rate as f32 * 1.1
        {
            facts["trimmed_from_seconds"] = json!(t.len() as f32 / rate as f32);
            t = audio::cut_to(&t, rate, s);
        }
        if t.len() as f32 / rate as f32 > 4.0 {
            warnings.push(format!("{:.1} s is long for an effect; pass --seconds", t.len() as f32 / rate as f32));
        }
        vec![("", audio::normalize(&t, Kind::Sfx, rate))]
    };
    let out_base = game.join(&job.out);
    if let Some(d) = out_base.parent() {
        std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
    }
    let tmp = std::env::temp_dir();
    let mut written = Vec::new();
    for (suffix, samples) in &outputs {
        let stem = out_base.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        let dest = out_base.with_file_name(format!("{stem}{suffix}.ogg"));
        let decoded = match audio::encode_ogg(samples, rate, job.quality, &dest, &tmp) {
            Ok(d) => d,
            Err(e) => {
                super::ensure_art_dir(game)?;
                let wav = game.join(format!("art/raw/{stem}{suffix}.processed.wav"));
                std::fs::write(&wav, audio::wav_bytes(samples, rate)).map_err(|e| e.to_string())?;
                return Err(format!("{e}\nthe processed audio is in {} (encode it with oggenc -q {} when you can)", wav.display(), job.quality));
            }
        };
        let is_loop = (job.kind == "music" && !job.once) || job.looping;
        let chk = audio::check(&decoded, rate, is_loop);
        let ogg = std::fs::read(&dest).map_err(|e| e.to_string())?;
        let mut f = facts.clone();
        f["seconds"] = json!(chk.secs);
        f["peak_db"] = json!(chk.peak_db);
        f["loud_rms_db"] = json!(chk.rms_db);
        if let Some(s) = chk.seam {
            f["seam_ratio"] = json!(s);
            if s > 4.0 {
                warnings.push(format!("{}: a click at the loop seam ({s:.1}×); regenerate or try other --bars", dest.display()));
            }
        }
        if !suffix.is_empty() {
            f["layer"] = json!(suffix.trim_start_matches('_'));
        }
        f["warnings"] = json!(warnings);
        let sc = Sidecar {
            generator: format!("mb {} {}", env!("CARGO_PKG_VERSION"), job.kind),
            date: super::now_utc(),
            provider: job.provider.clone(),
            model: job.model.clone(),
            via: via.clone(),
            prompt: job.subject.clone(),
            full_prompt: full_prompt.clone(),
            seed: job.seed,
            seed_honored: model.is_none_or(|m| m.seed),
            references: Vec::new(),
            style: style_ref.clone(),
            est_cost_usd: est,
            cost_usd: cost,
            raw: raw_rel.clone(),
            raw_sha256: super::sha256_hex(&bytes),
            output_sha256: super::sha256_hex(&ogg),
            allowed_names: verdict.allowed.clone(),
            model_text: text.clone(),
            recipe: serde_json::to_value(job).unwrap(),
            result: f,
            note: super::note_for(&job.provider),
        };
        super::write_sidecar(&dest, &sc)?;
        eprintln!("  wrote {} ({} KB): {chk}", dest.display(), ogg.len().div_ceil(1024));
        written.push(dest);
    }
    for w in &warnings {
        eprintln!("  warning: {w}");
    }
    if let Some(l) = facts["loop"].as_str() {
        eprintln!("  {l}");
    }
    if job.layers {
        eprintln!("  in the game: start both with play_looped in the same frame (the high layer at vol 0) and raise its vol with intensity");
    } else if job.kind == "music" && !job.once {
        eprintln!("  in the game: Sound::play_looped; for rhythm, schedule it with play_looped_at on the beat grid above (SPEC §5.6)");
    }
    eprintln!("  you can't hear it: check the numbers above (and the skill's check_audio.py) and listen on a phone before publishing");
    for w in &written {
        println!("{}", w.display());
    }
    Ok(())
}

/// Cuts the best whole-bar loop (or a free-length one without a steady beat).
fn make_loop(x: &[f32], rate: u32, job: &SoundJob, facts: &mut serde_json::Value, warnings: &mut Vec<String>) -> Result<Vec<f32>, String> {
    let secs = x.len() as f32 / rate as f32;
    let skip = (if secs > 20.0 { 1.5 } else { 0.5 } * rate as f32) as usize;
    let xfade = (0.03 * rate as f32) as usize;
    let tail = 1.0 * rate as f32; // leave the last second (fade-outs)
    let room = (x.len() as f32 - skip as f32 - tail).max(0.0) / rate as f32;
    let detected = if job.bpm.is_none() && job.seconds.is_none() { audio::detect_bpm(x, rate) } else { None };
    let beat = match (job.seconds, job.bpm, detected) {
        (Some(_), _, _) => None,
        (None, Some(b), _) => Some((b, "given".to_string())),
        (None, None, Some((b, conf))) if conf >= 0.08 => Some((b, format!("detected; confidence {conf:.2}"))),
        (None, None, d) => {
            warnings.push(format!(
                "no steady beat found{}; cut a free-length loop (pass --bpm if it has one)",
                d.map(|(b, c)| format!(" (best guess {b:.0} BPM, confidence {c:.2})")).unwrap_or_default()
            ));
            None
        }
    };
    let (start, len, score, label) = match beat {
        Some((bpm, how)) => {
            let bar = 4.0 * 60.0 / bpm;
            let mut bars = job.bars.unwrap_or(8).max(1);
            while bars > 1 && bars as f32 * bar > room * 0.9 {
                bars /= 2;
            }
            if (bars as f32 * bar) > room {
                return Err(format!("the track is {secs:.1} s, too short for a bar at {bpm:.0} BPM; pass --seconds"));
            }
            if job.bars.is_some_and(|b| b != bars) {
                warnings.push(format!("only {bars} bars fit in the {secs:.0} s source (asked for {})", job.bars.unwrap()));
            }
            let len = (bars as f32 * bar * rate as f32).round() as usize;
            let (s, l, score) = audio::find_loop(x, rate, len, len / 250, skip, xfade).ok_or("couldn't find a loop in the track")?;
            let final_bpm = bars as f32 * 4.0 * 60.0 * rate as f32 / l as f32;
            facts["bars"] = json!(bars);
            facts["bpm"] = json!(final_bpm);
            let label = format!(
                "loop: {:.3} s = {bars} bars of 4/4 at {final_bpm:.2} BPM ({how}), from {:.2} s into the source; match {score:.2}",
                l as f32 / rate as f32,
                s as f32 / rate as f32
            );
            (s, l, score, label)
        }
        None => {
            let want = job.seconds.unwrap_or(16.0).min(room);
            if want < 2.0 {
                return Err(format!("the track is {secs:.1} s, too short to loop"));
            }
            let len = (want * rate as f32) as usize;
            let (s, l, score) = audio::find_loop(x, rate, len, len / 20, skip, xfade).ok_or("couldn't find a loop in the track")?;
            let label = format!("loop: {:.3} s (free length), from {:.2} s into the source; match {score:.2}", l as f32 / rate as f32, s as f32 / rate as f32);
            (s, l, score, label)
        }
    };
    if score < 0.6 {
        warnings.push(format!("the best loop point is a weak match ({score:.2}); it may jump audibly. Regenerate or change --bars"));
    }
    facts["loop_start_seconds"] = json!(start as f32 / rate as f32);
    facts["loop_seconds"] = json!(len as f32 / rate as f32);
    facts["loop_samples"] = json!(len);
    facts["loop_match"] = json!(score);
    facts["loop"] = json!(label);
    Ok(audio::cut_loop(x, start, len, xfade))
}

pub fn regen(a: &super::RegenArgs, sc: &Sidecar) -> Result<(), String> {
    let mut job: SoundJob = serde_json::from_value(sc.recipe.clone()).map_err(|e| format!("sidecar recipe: {e}"))?;
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
            let media = if job.kind == "music" { Media::Music } else { Media::Sfx };
            job.model = provider::default_model(p, media).map(|m| m.id.to_string()).unwrap_or_default();
        }
    }
    if let Some(m) = &a.model {
        job.model = m.clone();
    }
    let raw = if a.reprocess {
        let p = game.join(&sc.raw);
        let b = std::fs::read(&p).map_err(|e| format!("{}: {e} (regenerate without --reprocess)", p.display()))?;
        let mime = match p.extension().and_then(|e| e.to_str()) {
            Some("mp3") => "audio/mpeg".to_string(),
            Some("ogg") => "audio/ogg".to_string(),
            Some("pcm") => std::fs::read_to_string(game.join(format!("{}.mime", sc.raw))).unwrap_or_else(|_| "audio/pcm;rate=24000".into()),
            _ => "audio/wav".to_string(),
        };
        Some((b, mime, sc.raw.clone()))
    } else {
        None
    };
    execute(&game, &job, a.dry_run, super::max_cost(a.max_cost), raw)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job() -> SoundJob {
        SoundJob {
            kind: "music".into(),
            name: "m".into(),
            subject: "dreamy harp".into(),
            out: "assets/m.ogg".into(),
            provider: "mock".into(),
            model: "mock-music".into(),
            seed: 1,
            bars: Some(4),
            bpm: None,
            seconds: None,
            layers: false,
            split_hz: 700.0,
            once: false,
            looping: false,
            negative: None,
            rate: 22050,
            quality: 4.0,
            no_style: false,
            allow_names: vec![],
        }
    }

    #[test]
    fn prompts() {
        let mut j = job();
        j.bpm = Some(120.0);
        let p = compose(&j, &Style::default());
        for want in ["dreamy harp.", "120 BPM", "Instrumental only", "loop seamlessly", "An original composition"] {
            assert!(p.contains(want), "missing {want:?} in {p}");
        }
        j.kind = "sfx".into();
        j.seconds = Some(0.4);
        j.looping = true;
        let p = compose(&j, &Style::default());
        assert!(p.contains("about 0.4 seconds") && p.contains("loops seamlessly"));
    }

    #[test]
    fn loop_from_mock_song_is_whole_bars() {
        let rate = 22050;
        let x = super::super::mock::song(112.0, 30.0, rate, 5);
        let mut facts = json!({});
        let mut warnings = Vec::new();
        let lp = make_loop(&x, rate, &job(), &mut facts, &mut warnings).unwrap();
        let bpm = facts["bpm"].as_f64().unwrap();
        assert!((bpm - 112.0).abs() < 1.5, "{bpm}");
        assert_eq!(facts["bars"], 4);
        assert!(audio::seam_ratio(&lp) < 4.0);
        assert!(warnings.is_empty(), "{warnings:?}");
    }
}
