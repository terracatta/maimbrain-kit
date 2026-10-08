//! Audio post-processing for `mb music` and `mb sfx`: decoding what a
//! provider returns (WAV, MP3, Ogg, raw PCM), mono at one sample rate,
//! tempo estimation, cutting a seamless whole-bar loop, splitting base/high
//! layers, loudness normalization with a look-ahead limiter (the same rules
//! as the skill's sfx.py: sfx ≈ −14 dBFS, music ≈ −20 dBFS RMS of the loud
//! part, peaks ≤ −3 dBFS before encoding, ≤ −1 dBFS decoded), and Ogg Vorbis
//! encoding with `oggenc`, checked by decoding the result.

use std::path::Path;
use std::process::Command;

pub const CEILING_DB: f32 = -3.0;
/// The limiter aims this far under CEILING_DB: Vorbis encoding adds a few
/// tenths of a dB to peaks, and the file should still read under −3.
pub const ENCODE_HEADROOM_DB: f32 = 0.5;
pub const DECODED_MAX_DB: f32 = -1.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Sfx,
    Music,
}

impl Kind {
    pub fn target_db(self) -> f32 {
        match self {
            Kind::Sfx => -14.0,
            Kind::Music => -20.0,
        }
    }
}

pub fn db(x: f32) -> f32 {
    20.0 * x.max(1e-9).log10()
}

pub fn undb(d: f32) -> f32 {
    10f32.powf(d / 20.0)
}

// --- decoding -------------------------------------------------------------------

/// Decodes audio bytes to (channels, sample rate). `mime` may be
/// "audio/pcm;rate=24000" for raw signed 16-bit little-endian mono.
pub fn decode(bytes: &[u8], mime: &str) -> Result<(Vec<Vec<f32>>, u32), String> {
    if let Some(rate) = mime.strip_prefix("audio/pcm;rate=") {
        let rate: u32 = rate.parse().map_err(|_| format!("bad PCM rate in {mime}"))?;
        let s = bytes.chunks_exact(2).map(|b| i16::from_le_bytes([b[0], b[1]]) as f32 / 32768.0).collect();
        return Ok((vec![s], rate));
    }
    use symphonia::core::codecs::audio::AudioDecoderOptions;
    use symphonia::core::formats::probe::Hint;
    use symphonia::core::formats::{FormatOptions, TrackType};
    use symphonia::core::io::MediaSourceStream;
    use symphonia::core::meta::MetadataOptions;
    let mut hint = Hint::new();
    match mime {
        "audio/mpeg" | "audio/mp3" => {
            hint.with_extension("mp3");
        }
        "audio/wav" | "audio/x-wav" | "audio/wave" => {
            hint.with_extension("wav");
        }
        "audio/ogg" => {
            hint.with_extension("ogg");
        }
        _ => {}
    }
    let mss = MediaSourceStream::new(Box::new(std::io::Cursor::new(bytes.to_vec())), Default::default());
    let mut format = symphonia::default::get_probe()
        .probe(&hint, mss, FormatOptions::default(), MetadataOptions::default())
        .map_err(|e| format!("can't read the audio ({mime}): {e}"))?;
    let track = format.default_track(TrackType::Audio).ok_or("no audio track")?;
    let params = track.codec_params.as_ref().and_then(|p| p.audio()).ok_or("no audio codec parameters")?.clone();
    let track_id = track.id;
    let mut decoder =
        symphonia::default::get_codecs().make_audio_decoder(&params, &AudioDecoderOptions::default()).map_err(|e| format!("can't decode the audio: {e}"))?;
    let mut chans: Vec<Vec<f32>> = Vec::new();
    let mut rate = params.sample_rate.unwrap_or(0);
    let mut inter: Vec<f32> = Vec::new();
    loop {
        let packet = match format.next_packet() {
            Ok(Some(p)) => p,
            Ok(None) => break,
            Err(_) => break,
        };
        if packet.track_id != track_id {
            continue;
        }
        match decoder.decode(&packet) {
            Ok(buf) => {
                let spec = buf.spec();
                let n = spec.channels().count().max(1);
                rate = spec.rate();
                inter.resize(buf.samples_interleaved(), 0.0);
                buf.copy_to_slice_interleaved(&mut inter);
                if chans.is_empty() {
                    chans = vec![Vec::new(); n];
                }
                for frame in inter.chunks_exact(n) {
                    for (c, v) in frame.iter().enumerate() {
                        if c < chans.len() {
                            chans[c].push(*v);
                        }
                    }
                }
            }
            Err(symphonia::core::errors::Error::DecodeError(_)) => continue,
            Err(e) => return Err(format!("decoding: {e}")),
        }
    }
    if chans.is_empty() || chans[0].is_empty() || rate == 0 {
        return Err("the audio decoded to nothing".into());
    }
    Ok((chans, rate))
}

pub fn to_mono(chans: &[Vec<f32>]) -> Vec<f32> {
    let n = chans.iter().map(Vec::len).min().unwrap_or(0);
    let k = 1.0 / chans.len().max(1) as f32;
    (0..n).map(|i| chans.iter().map(|c| c[i]).sum::<f32>() * k).collect()
}

/// Windowed-sinc resampling (Blackman window, 32 zero crossings).
pub fn resample(x: &[f32], from: u32, to: u32) -> Vec<f32> {
    if from == to || x.is_empty() {
        return x.to_vec();
    }
    let ratio = to as f64 / from as f64;
    let cutoff = ratio.min(1.0) * 0.95;
    let half = 16.0 / cutoff.min(1.0);
    let n_out = (x.len() as f64 * ratio).round() as usize;
    let mut out = Vec::with_capacity(n_out);
    for o in 0..n_out {
        let t = o as f64 / ratio;
        let lo = (t - half).ceil().max(0.0) as usize;
        let hi = ((t + half).floor() as usize).min(x.len() - 1);
        let mut acc = 0.0;
        let mut wsum = 0.0;
        for (i, &xi) in x.iter().enumerate().take(hi + 1).skip(lo) {
            let d = i as f64 - t;
            let s = if d.abs() < 1e-12 { 1.0 } else { (std::f64::consts::PI * d * cutoff).sin() / (std::f64::consts::PI * d * cutoff) };
            let wpos = (d / half + 1.0) / 2.0; // 0..1 across the window
            let w = 0.42 - 0.5 * (2.0 * std::f64::consts::PI * wpos).cos() + 0.08 * (4.0 * std::f64::consts::PI * wpos).cos();
            let k = s * w;
            acc += xi as f64 * k;
            wsum += k;
        }
        out.push(if wsum.abs() > 1e-9 { (acc / wsum) as f32 } else { 0.0 });
    }
    out
}

// --- levels ---------------------------------------------------------------------

/// RMS of the loudest two thirds of the samples (ignores near-silence), as sfx.py does.
pub fn loud_rms(x: &[f32]) -> f32 {
    if x.is_empty() {
        return 0.0;
    }
    let mut sq: Vec<f32> = x.iter().map(|v| v * v).collect();
    sq.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let top = &sq[sq.len() / 3..];
    (top.iter().map(|&v| v as f64).sum::<f64>() / top.len().max(1) as f64).sqrt() as f32
}

pub fn peak(x: &[f32]) -> f32 {
    x.iter().fold(0.0, |m, v| m.max(v.abs()))
}

pub fn gain_for(x: &[f32], kind: Kind) -> f32 {
    undb(kind.target_db()) / loud_rms(x).max(1e-9)
}

/// Applies gain g, then a look-ahead peak limiter (a smooth gain envelope that
/// is already down when a peak arrives and recovers over ~80 ms), so no sample
/// exceeds CEILING_DB and no distortion is added. Port of sfx.py's `_limit`.
pub fn limit(x: &[f32], g: f32, rate: u32) -> Vec<f32> {
    let ceiling = undb(CEILING_DB - ENCODE_HEADROOM_DB);
    let y: Vec<f32> = x.iter().map(|v| v * g).collect();
    let n = y.len();
    let need: Vec<f32> = y.iter().map(|v| if v.abs() > ceiling { ceiling / v.abs() } else { 1.0 }).collect();
    let la = ((0.0015 * rate as f32) as usize).max(1);
    // m[i] = min(need[i-la ..= i+la]) via a monotonic deque.
    let mut m = vec![1.0f32; n];
    let mut q: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
    for j in 0..n + la {
        if j < n {
            while q.back().is_some_and(|&b| need[b] >= need[j]) {
                q.pop_back();
            }
            q.push_back(j);
        }
        if j >= la {
            let i = j - la;
            while q.front().is_some_and(|&f| f + la < i) {
                q.pop_front();
            }
            m[i] = need[*q.front().unwrap()];
        }
    }
    let mut acc = vec![0.0f64; n + 1];
    for i in 0..n {
        acc[i + 1] = acc[i] + m[i] as f64;
    }
    let k = 1.0 - (-1.0 / (0.08 * rate as f32)).exp();
    let mut env = 1.0f32;
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let lo = i.saturating_sub(la);
        let hi = (i + la + 1).min(n);
        let a = ((acc[hi] - acc[lo]) / (hi - lo) as f64) as f32;
        env = a.min(env + (1.0 - env) * k);
        out.push(y[i] * env);
    }
    out
}

pub fn normalize(x: &[f32], kind: Kind, rate: u32) -> Vec<f32> {
    limit(x, gain_for(x, kind), rate)
}

// --- trimming (sfx) -------------------------------------------------------------

/// Trims leading and trailing near-silence (below `floor_db` relative to the
/// peak), keeping 3 ms before the onset, then fades the last 8 ms.
/// Cuts `x` to at most `secs` seconds with a short fade-out (models often
/// return more than was asked for, e.g. 1.2 s for a 0.6 s effect).
pub fn cut_to(x: &[f32], rate: u32, secs: f32) -> Vec<f32> {
    let n = ((secs.max(0.05) * rate as f32) as usize).min(x.len());
    let mut y = x[..n].to_vec();
    let fade = ((0.015 * rate as f32) as usize).min(n / 2).max(1);
    for i in 0..fade {
        y[n - 1 - i] *= i as f32 / fade as f32;
    }
    y
}

pub fn trim_silence(x: &[f32], rate: u32, floor_db: f32) -> Vec<f32> {
    let p = peak(x);
    if p <= 0.0 {
        return x.to_vec();
    }
    let thr = p * undb(floor_db);
    let first = x.iter().position(|v| v.abs() > thr).unwrap_or(0);
    let last = x.iter().rposition(|v| v.abs() > thr).unwrap_or(x.len() - 1);
    let pre = (0.003 * rate as f32) as usize;
    let post = (0.03 * rate as f32) as usize;
    let mut y = x[first.saturating_sub(pre)..(last + post).min(x.len())].to_vec();
    let f = ((0.008 * rate as f32) as usize).min(y.len() / 2);
    let n = y.len();
    for i in 0..f {
        y[n - 1 - i] *= i as f32 / f as f32;
    }
    y
}

// --- tempo and loops -------------------------------------------------------------

/// An onset-strength envelope at 100 frames per second.
fn onset_envelope(x: &[f32], rate: u32) -> Vec<f32> {
    let hop = (rate / 100) as usize;
    let win = hop * 2;
    let mut energy = Vec::new();
    let mut i = 0;
    // Emphasize transients: first difference before energy.
    while i + win <= x.len() {
        let e: f32 = (i + 1..i + win).map(|j| (x[j] - 0.97 * x[j - 1]).powi(2)).sum::<f32>() / win as f32;
        energy.push((e + 1e-10).ln());
        i += hop;
    }
    let mut flux: Vec<f32> = (0..energy.len()).map(|k| if k == 0 { 0.0 } else { (energy[k] - energy[k - 1]).max(0.0) }).collect();
    // Remove the local mean so sustained parts don't dominate.
    let w = 20;
    let mut out = vec![0.0; flux.len()];
    for k in 0..flux.len() {
        let lo = k.saturating_sub(w);
        let hi = (k + w + 1).min(flux.len());
        let mean = flux[lo..hi].iter().sum::<f32>() / (hi - lo) as f32;
        out[k] = (flux[k] - mean).max(0.0);
    }
    flux.clear();
    out
}

/// Estimated tempo in BPM (60–200) and a confidence 0..1, from the
/// autocorrelation of the onset envelope.
pub fn detect_bpm(x: &[f32], rate: u32) -> Option<(f32, f32)> {
    let env = onset_envelope(x, rate);
    if env.len() < 400 {
        return None;
    }
    let fps = 100.0f32;
    let ac = |lag: usize| -> f32 { env.iter().zip(&env[lag..]).map(|(a, b)| a * b).sum::<f32>() / (env.len() - lag) as f32 };
    let zero = ac(0).max(1e-12);
    let (lo, hi) = ((fps * 60.0 / 200.0) as usize, (fps * 60.0 / 60.0) as usize);
    let mut best = (0usize, f32::MIN);
    let mut vals = vec![0.0; hi + 2];
    for (lag, v) in vals.iter_mut().enumerate().take(hi + 2).skip(lo.saturating_sub(1)) {
        *v = ac(lag);
    }
    for lag in lo..=hi {
        let bpm = 60.0 * fps / lag as f32;
        // Prefer tempos near 120 (a log-Gaussian), and reward periodicity at 2× the lag too.
        let pref = (-0.5 * ((bpm / 120.0).log2() / 0.9).powi(2)).exp();
        let two = if lag * 2 < env.len() / 2 { ac(lag * 2) } else { 0.0 };
        let score = (vals[lag] + 0.5 * two) * (0.6 + 0.4 * pref);
        if score > best.1 {
            best = (lag, score);
        }
    }
    let lag = best.0;
    if lag == 0 {
        return None;
    }
    // Parabolic refinement.
    let (a, b, c) = (vals[lag - 1], vals[lag], vals[lag + 1]);
    let denom = a - 2.0 * b + c;
    let off = if denom.abs() > 1e-12 { 0.5 * (a - c) / denom } else { 0.0 };
    let period = lag as f32 + off.clamp(-0.5, 0.5);
    let conf = (vals[lag] / zero).clamp(0.0, 1.0);
    Some((60.0 * fps / period, conf))
}

fn ncc(a: &[f32], b: &[f32]) -> f32 {
    let (mut ab, mut aa, mut bb) = (0.0f64, 0.0f64, 0.0f64);
    for (x, y) in a.iter().zip(b) {
        ab += (*x as f64) * (*y as f64);
        aa += (*x as f64).powi(2);
        bb += (*y as f64).powi(2);
    }
    if aa < 1e-12 || bb < 1e-12 { 0.0 } else { (ab / (aa * bb).sqrt()) as f32 }
}

fn env_rms(x: &[f32], hop: usize) -> Vec<f32> {
    x.chunks(hop).map(|c| (c.iter().map(|v| v * v).sum::<f32>() / c.len() as f32).sqrt()).collect()
}

/// Finds the best loop of `len` samples (± `slack` samples) in x: where
/// the audio just after the end best matches the audio just after the start,
/// both in loudness contour (~1 s) and waveform (~20 ms). Skips the first
/// `skip` samples (intros fade in) and leaves room for a crossfade.
pub fn find_loop(x: &[f32], rate: u32, len: usize, slack: usize, skip: usize, xfade: usize) -> Option<(usize, usize, f32)> {
    let ctx = (rate as usize).min(len / 4); // contour context
    if x.len() < skip + len + slack + ctx.max(xfade) + 1 {
        return None;
    }
    let hop = (rate / 100) as usize;
    let env = env_rms(x, hop);
    let last_start = x.len() - len - slack - ctx.max(xfade) - 1;
    let mut cands: Vec<(f32, usize)> = Vec::new();
    let mut s = skip;
    let ctx_frames = ctx / hop;
    while s <= last_start {
        let e = s + len;
        let (fs, fe) = (s / hop, e / hop);
        if fe + ctx_frames >= env.len() {
            break;
        }
        let a = &env[fs..fs + ctx_frames];
        let b = &env[fe..fe + ctx_frames];
        let mean_a = a.iter().sum::<f32>() / a.len() as f32;
        let mean_b = b.iter().sum::<f32>() / b.len() as f32;
        let level = (mean_a.min(mean_b) / mean_a.max(mean_b).max(1e-9)).clamp(0.0, 1.0);
        let shape = ncc(a, b).max(0.0);
        cands.push((0.6 * shape + 0.4 * level, s));
        s += hop;
    }
    cands.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let w = (0.02 * rate as f32) as usize;
    let mut best: Option<(usize, usize, f32)> = None;
    for &(coarse, s) in cands.iter().take(12) {
        // Fine-align the end at sample level within ±slack (at least ±1 period of 50 Hz).
        let span = slack.max((rate / 50) as usize);
        let a = &x[s..s + w];
        let mut bl = len;
        let mut bc = f32::MIN;
        let lo = len.saturating_sub(span);
        for l in lo..=len + span {
            if s + l + w > x.len() {
                break;
            }
            let c = ncc(a, &x[s + l..s + l + w]);
            if c > bc {
                bc = c;
                bl = l;
            }
        }
        let score = 0.5 * coarse + 0.5 * bc.max(0.0);
        if best.is_none_or(|b| score > b.2) {
            best = Some((s, bl, score));
        }
    }
    best
}

/// The loop region with its seam cross-faded: the first `xfade` samples blend
/// from what followed the end in the source into the start, so playing the
/// result on repeat is continuous.
pub fn cut_loop(x: &[f32], start: usize, len: usize, xfade: usize) -> Vec<f32> {
    let mut out = x[start..start + len].to_vec();
    let xf = xfade.min(len / 2).min(x.len().saturating_sub(start + len));
    for i in 0..xf {
        let t = (i as f32 + 0.5) / xf as f32;
        let t = t * t * (3.0 - 2.0 * t);
        out[i] = x[start + i] * t + x[start + len + i] * (1.0 - t);
    }
    out
}

/// Jump at the loop seam relative to the typical sample-to-sample change
/// (check_audio.py's measure: above ~4× is an audible click).
pub fn seam_ratio(x: &[f32]) -> f32 {
    if x.len() < 3 {
        return 0.0;
    }
    // Compared with the sample-to-sample change right around the seam (the
    // last and first 256 samples), so a loop that wraps inside a noisy hi-hat
    // isn't mistaken for a click.
    let seam = (x[0] - x[x.len() - 1]).abs();
    let n = (x.len() / 2).min(256);
    let head = (1..n).map(|i| (x[i] - x[i - 1]).abs());
    let tail = (x.len() - n + 1..x.len()).map(|i| (x[i] - x[i - 1]).abs());
    let typical = head.chain(tail).fold(0.0f32, |a, v| a + v) / (2 * (n - 1)).max(1) as f32;
    seam / typical.max(1e-9)
}

// --- layers -----------------------------------------------------------------------

/// A Linkwitz-Riley low-pass (two cascaded Butterworth biquads), run over the
/// loop twice so its state at the seam is the steady state (seamless).
pub fn lowpass_loop(x: &[f32], rate: u32, cutoff: f32) -> Vec<f32> {
    let w0 = 2.0 * std::f32::consts::PI * cutoff / rate as f32;
    let (sin, cos) = w0.sin_cos();
    let alpha = sin / (2.0 * std::f32::consts::FRAC_1_SQRT_2);
    let a0 = 1.0 + alpha;
    let b0 = (1.0 - cos) / 2.0 / a0;
    let b1 = (1.0 - cos) / a0;
    let b2 = b0;
    let a1 = -2.0 * cos / a0;
    let a2 = (1.0 - alpha) / a0;
    let mut st = [[0.0f32; 4]; 2]; // x1 x2 y1 y2 per stage
    let mut out = vec![0.0; x.len()];
    for pass in 0..2 {
        for (i, &v) in x.iter().enumerate() {
            let mut s = v;
            for z in st.iter_mut() {
                let y = b0 * s + b1 * z[0] + b2 * z[1] - a1 * z[2] - a2 * z[3];
                z[1] = z[0];
                z[0] = s;
                z[3] = z[2];
                z[2] = y;
                s = y;
            }
            if pass == 1 {
                out[i] = s;
            }
        }
    }
    out
}

/// Splits a loop into a calm base (low-passed) and a high layer (the rest):
/// base + high = the full mix, so a game plays base alone and fades the high
/// layer in as intensity rises.
pub fn split_layers(x: &[f32], rate: u32, cutoff: f32) -> (Vec<f32>, Vec<f32>) {
    let base = lowpass_loop(x, rate, cutoff);
    let high = x.iter().zip(&base).map(|(a, b)| a - b).collect();
    (base, high)
}

// --- files --------------------------------------------------------------------------

pub fn wav_bytes(x: &[f32], rate: u32) -> Vec<u8> {
    let data_len = (x.len() * 2) as u32;
    let mut b = Vec::with_capacity(44 + x.len() * 2);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data_len).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes()); // PCM
    b.extend_from_slice(&1u16.to_le_bytes()); // mono
    b.extend_from_slice(&rate.to_le_bytes());
    b.extend_from_slice(&(rate * 2).to_le_bytes());
    b.extend_from_slice(&2u16.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data_len.to_le_bytes());
    for v in x {
        b.extend_from_slice(&((v.clamp(-1.0, 1.0) * 32767.0).round() as i16).to_le_bytes());
    }
    b
}

pub fn has_oggenc() -> bool {
    Command::new("oggenc").arg("--version").output().is_ok_and(|o| o.status.success())
}

pub fn oggenc_hint() -> &'static str {
    if cfg!(target_os = "macos") {
        "brew install vorbis-tools"
    } else if cfg!(windows) {
        "get oggenc for Windows (e.g. rarewares.org) and put it on PATH"
    } else {
        "sudo apt install vorbis-tools"
    }
}

/// Encodes mono samples to Ogg Vorbis at `out` with oggenc -q `quality`,
/// then decodes the file and, if Vorbis pushed a peak past DECODED_MAX_DB,
/// scales down and encodes again. Returns the decoded samples.
pub fn encode_ogg(x: &[f32], rate: u32, quality: f32, out: &Path, tmp: &Path) -> Result<Vec<f32>, String> {
    if !has_oggenc() {
        return Err(format!("oggenc not found ({}); the processed WAV is kept in art/raw/", oggenc_hint()));
    }
    if peak(x) < 1e-4 {
        return Err(format!("{}: the sound is silent", out.display()));
    }
    let limit = undb(DECODED_MAX_DB);
    let mut s = x.to_vec();
    let wav = tmp.join(format!("mb-{}-{}.wav", std::process::id(), out.file_stem().and_then(|s| s.to_str()).unwrap_or("x")));
    for _ in 0..4 {
        std::fs::write(&wav, wav_bytes(&s, rate)).map_err(|e| format!("{}: {e}", wav.display()))?;
        let st =
            Command::new("oggenc").args(["-Q", "-q", &format!("{quality}"), "-o"]).arg(out).arg(&wav).status().map_err(|e| format!("running oggenc: {e}"))?;
        if !st.success() {
            let _ = std::fs::remove_file(&wav);
            return Err("oggenc failed".into());
        }
        let bytes = std::fs::read(out).map_err(|e| e.to_string())?;
        let (ch, _) = decode(&bytes, "audio/ogg")?;
        let dec = to_mono(&ch);
        let p = peak(&dec);
        if p <= limit {
            let _ = std::fs::remove_file(&wav);
            return Ok(dec);
        }
        let k = limit / p * 0.97;
        s.iter_mut().for_each(|v| *v *= k);
    }
    let _ = std::fs::remove_file(&wav);
    Err(format!("{}: decoded peak still above {DECODED_MAX_DB} dBFS", out.display()))
}

/// What check_audio.py reports, for the log.
pub struct Check {
    pub secs: f32,
    pub peak_db: f32,
    pub rms_db: f32,
    pub seam: Option<f32>,
}

pub fn check(x: &[f32], rate: u32, is_loop: bool) -> Check {
    Check { secs: x.len() as f32 / rate as f32, peak_db: db(peak(x)), rms_db: db(loud_rms(x)), seam: is_loop.then(|| seam_ratio(x)) }
}

impl std::fmt::Display for Check {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:.2} s · peak {:.1} dB · loud RMS {:.1} dB", self.secs, self.peak_db, self.rms_db)?;
        if let Some(s) = self.seam {
            write!(f, " · loop seam {:.1}× {}", s, if s > 4.0 { "(CLICK)" } else { "(clean)" })?;
        }
        Ok(())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    /// A test "song": a kick on every beat, a hat on off-beats, a chord
    /// progression changing every bar, after a 1 s fade-in.
    pub fn song(bpm: f32, secs: f32, rate: u32) -> Vec<f32> {
        let n = (secs * rate as f32) as usize;
        let beat = 60.0 / bpm;
        let chords = [[220.0, 277.18, 329.63], [196.0, 246.94, 293.66], [174.61, 220.0, 261.63], [196.0, 246.94, 293.66]];
        let mut seed = 12345u32;
        (0..n)
            .map(|i| {
                let t = i as f32 / rate as f32;
                let bt = t % beat;
                let kick = (-bt * 30.0).exp() * (2.0 * std::f32::consts::PI * (50.0 + 80.0 * (-bt * 40.0).exp()) * bt).sin();
                let ht = (t + beat / 2.0) % beat;
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                let noise = (seed >> 9) as f32 / (1u32 << 23) as f32 * 2.0 - 1.0;
                let hat = (-ht * 80.0).exp() * noise * 0.3;
                let bar = ((t / (beat * 4.0)) as usize) % 4;
                let pad: f32 = chords[bar].iter().map(|f| (2.0 * std::f32::consts::PI * f * t).sin()).sum::<f32>() * 0.08;
                let fade = (t / 1.0).min(1.0);
                (kick * 0.6 + hat + pad) * fade
            })
            .collect()
    }

    #[test]
    fn wav_roundtrip_through_symphonia() {
        let x: Vec<f32> = (0..4410).map(|i| (i as f32 * 0.05).sin() * 0.5).collect();
        let (ch, rate) = decode(&wav_bytes(&x, 22050), "audio/wav").unwrap();
        assert_eq!(rate, 22050);
        assert_eq!(ch[0].len(), x.len());
        assert!((ch[0][100] - x[100]).abs() < 1e-3);
    }

    #[test]
    fn raw_pcm() {
        let b: Vec<u8> = [0i16, 16384, -16384].iter().flat_map(|v| v.to_le_bytes()).collect();
        let (ch, rate) = decode(&b, "audio/pcm;rate=24000").unwrap();
        assert_eq!(rate, 24000);
        assert_eq!(ch[0], vec![0.0, 0.5, -0.5]);
    }

    #[test]
    fn resample_keeps_pitch_and_length() {
        let rate = 48000;
        let x: Vec<f32> = (0..rate).map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / rate as f32).sin()).collect();
        let y = resample(&x[..12000], 48000, 44100);
        assert_eq!(y.len(), 11025);
        // Zero crossings ≈ 2 × 440 × 0.25 s.
        let zc = y.windows(2).filter(|w| w[0] <= 0.0 && w[1] > 0.0).count();
        assert!((108..=112).contains(&zc), "{zc}");
    }

    #[test]
    fn normalize_hits_targets_without_clipping() {
        let rate = 44100;
        let x: Vec<f32> = (0..rate).map(|i| (i as f32 * 0.03).sin() * 0.05 + if i == 20000 { 0.9 } else { 0.0 }).collect();
        for kind in [Kind::Sfx, Kind::Music] {
            let y = normalize(&x, kind, rate as u32);
            assert!(db(peak(&y)) <= CEILING_DB - ENCODE_HEADROOM_DB + 0.01);
            assert!((db(loud_rms(&y)) - kind.target_db()).abs() < 1.5, "{kind:?}: {}", db(loud_rms(&y)));
        }
    }

    #[test]
    fn trims_silence() {
        let rate = 44100u32;
        let mut x = vec![0.0f32; 4410];
        x.extend((0..4410).map(|i| (i as f32 * 0.1).sin() * 0.5));
        x.extend(vec![0.0; 8820]);
        let y = trim_silence(&x, rate, -50.0);
        assert!(y.len() < 4410 + 1500 && y.len() > 4410, "{}", y.len());
    }

    #[test]
    fn detects_tempo() {
        for bpm in [96.0, 120.0, 140.0] {
            let x = song(bpm, 20.0, 22050);
            let (got, _) = detect_bpm(&x, 22050).unwrap();
            assert!((got - bpm).abs() / bpm < 0.02, "{bpm} → {got}");
        }
    }

    #[test]
    fn finds_whole_bar_loop_with_clean_seam() {
        let rate = 22050;
        let bpm = 120.0;
        let x = song(bpm, 24.0, rate);
        let bars = 4;
        let len = (bars as f32 * 4.0 * 60.0 / bpm * rate as f32) as usize;
        let (s, l, score) = find_loop(&x, rate, len, len / 200, rate as usize, 1000).unwrap();
        assert!(score > 0.8, "{score}");
        assert!((l as i64 - len as i64).abs() <= (len / 200) as i64 + (rate / 50) as i64);
        assert!(s >= rate as usize, "skips the fade-in");
        let lp = cut_loop(&x, s, l, 1000);
        assert!(seam_ratio(&lp) < 4.0, "{}", seam_ratio(&lp));
    }

    #[test]
    fn layers_sum_to_full_mix() {
        let x = song(120.0, 4.0, 22050);
        let (b, h) = split_layers(&x, 22050, 800.0);
        assert!(b.iter().zip(&h).zip(&x).all(|((b, h), x)| (b + h - x).abs() < 1e-5));
        assert!(loud_rms(&b) > 0.01 && loud_rms(&h) > 0.01);
    }

    #[test]
    fn ogg_encode_if_available() {
        if !has_oggenc() {
            eprintln!("skipping: oggenc not installed");
            return;
        }
        let dir = std::env::temp_dir().join(format!("mb-ogg-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let x = normalize(&song(120.0, 2.0, 44100), Kind::Music, 44100);
        let out = dir.join("t.ogg");
        let dec = encode_ogg(&x, 44100, 4.0, &out, &dir).unwrap();
        assert!(db(peak(&dec)) <= DECODED_MAX_DB + 0.01);
        assert!((dec.len() as i64 - x.len() as i64).abs() < 64);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn cut_to_trims_long_effects_and_fades_the_end() {
        let rate = 1000;
        let x = vec![0.5f32; 1200];
        let y = cut_to(&x, rate, 0.6);
        assert_eq!(y.len(), 600);
        assert_eq!(y[599], 0.0);
        assert!(y[500] > 0.4);
        assert_eq!(cut_to(&x[..300], rate, 0.6).len(), 300, "shorter than asked: untouched");
    }
}
