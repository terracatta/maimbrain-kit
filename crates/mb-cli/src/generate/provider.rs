//! The provider layer: what a generation request and result look like, the
//! model catalog with documented prices, an HTTP transport that tests can
//! replace, and `pick`, which turns a provider name into an implementation.
//!
//! Providers: `gemini` (Google's Gemini API: Nano Banana images, Lyria
//! music), `vertex` (the same Google models through Vertex AI, plus
//! lyria-002), `openai` (GPT Image), `elevenlabs` (sound effects, music) and
//! `mock` (offline and deterministic, for trying the pipeline and for tests).

use std::time::Duration;

use super::audio::Kind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Media {
    Image,
    Music,
    Sfx,
}

#[derive(Clone, Copy, Debug)]
pub enum Price {
    /// Per image, by resolution tier ("0.5K", "1K", "2K", "4K").
    PerImage(&'static [(&'static str, f64)]),
    /// Per call (a clip or a song).
    PerCall(f64),
    /// Per minute of audio requested.
    PerMinute(f64),
    /// Billed by image output tokens: (USD per 1M tokens, rough tokens per 1K image at low/medium/high).
    ImageTokens(f64, [u32; 3]),
    Free,
}

pub struct Model {
    pub provider: &'static str,
    pub id: &'static str,
    pub media: Media,
    pub price: Price,
    pub seed: bool,
    pub default: bool,
    pub note: &'static str,
}

/// Prices from the providers' pricing pages as of October 2026:
/// https://ai.google.dev/gemini-api/docs/pricing,
/// https://cloud.google.com/vertex-ai/generative-ai/pricing,
/// https://developers.openai.com/api/docs/pricing, https://elevenlabs.io/pricing/api
pub const MODELS: &[Model] = &[
    Model {
        provider: "gemini",
        id: "gemini-nano-banana-2.1",
        media: Media::Image,
        price: Price::PerImage(&[("1K", 0.0336), ("2K", 0.0504), ("4K", 0.113)]),
        seed: true,
        default: true,
        note: "Nano Banana 2.1: the workhorse; style references, edits, good text",
    },
    Model {
        provider: "gemini",
        id: "gemini-3.1-flash-lite-image",
        media: Media::Image,
        price: Price::PerImage(&[("1K", 0.0336)]),
        seed: true,
        default: false,
        note: "Nano Banana 2 Lite: fastest; 1K only; weak with several references",
    },
    Model {
        provider: "gemini",
        id: "gemini-3.1-flash-image",
        media: Media::Image,
        price: Price::PerImage(&[("0.5K", 0.045), ("1K", 0.067), ("2K", 0.101), ("4K", 0.151)]),
        seed: true,
        default: false,
        note: "Nano Banana 2: the only one with 512 px output",
    },
    Model {
        provider: "gemini",
        id: "gemini-3-pro-image",
        media: Media::Image,
        price: Price::PerImage(&[("1K", 0.134), ("2K", 0.134), ("4K", 0.24)]),
        seed: true,
        default: false,
        note: "Nano Banana Pro: best for hard briefs (many references, consistency); 4× the price",
    },
    Model {
        provider: "gemini",
        id: "lyria-3-clip-preview",
        media: Media::Music,
        price: Price::PerCall(0.04),
        seed: true,
        default: true,
        note: "Lyria 3 Clip: always 30 s, MP3 (44.1 kHz stereo); loops are cut from it",
    },
    Model {
        provider: "gemini",
        id: "lyria-3.5",
        media: Media::Music,
        price: Price::PerCall(0.08),
        seed: true,
        default: false,
        note: "Lyria 3.5: full songs (length set in the prompt), WAV; for longer loops",
    },
    Model {
        provider: "vertex",
        id: "gemini-nano-banana-2.1",
        media: Media::Image,
        price: Price::PerImage(&[("1K", 0.0336), ("2K", 0.0504), ("4K", 0.113)]),
        seed: true,
        default: true,
        note: "the Gemini image model through Vertex AI (Google Cloud credits apply)",
    },
    Model {
        provider: "vertex",
        id: "lyria-3-clip-preview",
        media: Media::Music,
        price: Price::PerCall(0.04),
        seed: true,
        default: true,
        note: "Lyria 3 Clip through Vertex AI",
    },
    Model {
        provider: "vertex",
        id: "lyria-002",
        media: Media::Music,
        price: Price::PerCall(0.06),
        seed: true,
        default: false,
        note: "Lyria 2: 30 s instrumental WAV at 48 kHz; honors seed and a negative prompt",
    },
    Model {
        provider: "openai",
        id: "gpt-image-2.5-flare",
        media: Media::Image,
        price: Price::ImageTokens(30.0, [272, 1056, 4160]),
        seed: false,
        default: true,
        note: "GPT Image 2.5 Flare: native transparent backgrounds; no seed",
    },
    Model {
        provider: "openai",
        id: "gpt-image-2.5-sunburst",
        media: Media::Image,
        price: Price::ImageTokens(30.0, [272, 1056, 4160]),
        seed: false,
        default: false,
        note: "GPT Image 2.5 Sunburst: most precise edits; no seed",
    },
    Model {
        provider: "elevenlabs",
        id: "eleven_text_to_sound_v2",
        media: Media::Sfx,
        price: Price::PerMinute(0.12),
        seed: false,
        default: true,
        note: "sound effects 0.5–30 s, can loop; free plan includes a few a month",
    },
    Model {
        provider: "elevenlabs",
        id: "music_v2_5",
        media: Media::Music,
        price: Price::PerMinute(0.15),
        seed: false,
        default: true,
        note: "Eleven Music: exact length, guaranteed instrumental; commercial use on paid plans",
    },
    Model {
        provider: "mock",
        id: "mock-image",
        media: Media::Image,
        price: Price::Free,
        seed: true,
        default: true,
        note: "offline, deterministic placeholder art",
    },
    Model { provider: "mock", id: "mock-music", media: Media::Music, price: Price::Free, seed: true, default: true, note: "offline, deterministic test song" },
    Model { provider: "mock", id: "mock-sfx", media: Media::Sfx, price: Price::Free, seed: true, default: true, note: "offline, deterministic test effect" },
];

pub fn find_model(provider: &str, id: &str) -> Option<&'static Model> {
    MODELS.iter().find(|m| m.provider == provider && m.id == id)
}

pub fn default_model(provider: &str, media: Media) -> Option<&'static Model> {
    MODELS.iter().find(|m| m.provider == provider && m.media == media && m.default)
}

/// Estimated USD for one call.
pub fn estimate(model: &Model, resolution: &str, quality: usize, seconds: f32) -> Option<f64> {
    match model.price {
        Price::PerImage(tiers) => tiers.iter().find(|t| t.0 == resolution).or(tiers.first()).map(|t| t.1),
        Price::PerCall(p) => Some(p),
        Price::PerMinute(p) => Some(p * (seconds.max(1.0) as f64) / 60.0),
        Price::ImageTokens(per_m, toks) => {
            let scale = match resolution {
                "2K" => 2.5,
                "4K" => 6.0,
                _ => 1.0,
            };
            Some(per_m * toks[quality.min(2)] as f64 * scale / 1e6)
        }
        Price::Free => Some(0.0),
    }
}

pub fn price_label(model: &Model) -> String {
    match model.price {
        Price::PerImage(t) => t.iter().map(|(r, p)| format!("{r} ${p:.4}")).collect::<Vec<_>>().join(", ") + " per image",
        Price::PerCall(p) => format!("${p:.2} per call"),
        Price::PerMinute(p) => format!("${p:.2} per minute"),
        Price::ImageTokens(p, t) => format!("${p:.0}/1M image tokens (≈ ${:.3}–${:.3} per 1K image)", p * t[0] as f64 / 1e6, p * t[2] as f64 / 1e6),
        Price::Free => "free".into(),
    }
}

// --- requests -----------------------------------------------------------------

#[derive(Clone, Debug)]
pub struct RefImage {
    pub bytes: Vec<u8>,
    pub mime: String,
}

/// What the mock provider needs to draw a plausible stand-in (real providers ignore it).
#[derive(Clone, Debug, Default)]
pub struct MockHint {
    pub kind: String,
    pub frames: u32,
    pub key: Option<[u8; 3]>,
    pub palette: Vec<[u8; 3]>,
    pub pixel_art: bool,
}

#[derive(Clone, Debug)]
pub struct ImageRequest {
    pub model: String,
    pub prompt: String,
    /// Style references first, then subject references (photos to restyle).
    pub refs: Vec<RefImage>,
    pub aspect: String,
    pub resolution: String,
    /// Ask for a transparent background where the model can make one.
    pub transparent: bool,
    pub seed: Option<u64>,
    /// 0 low, 1 medium, 2 high (OpenAI).
    pub quality: usize,
    pub hint: MockHint,
}

#[derive(Clone, Debug)]
pub struct AudioRequest {
    pub model: String,
    pub prompt: String,
    pub kind: Kind,
    pub seconds: Option<f32>,
    pub looping: bool,
    pub seed: Option<u64>,
    pub negative: Option<String>,
}

pub struct Output {
    pub bytes: Vec<u8>,
    pub mime: String,
    /// Any text the model sent back (commentary, lyrics, structure).
    pub text: Option<String>,
    /// The actual cost when the response says (token usage).
    pub cost_usd: Option<f64>,
}

pub trait Provider {
    fn name(&self) -> &'static str;
    fn image(&self, req: &ImageRequest) -> Result<Output, String>;
    fn audio(&self, req: &AudioRequest) -> Result<Output, String>;
    /// A line to print after a call (Maimbrain's keys: the month's usage).
    fn note(&self) -> Option<String> {
        None
    }
    /// Set when the call went through someone else's keys ("maimbrain"); recorded in the sidecar.
    fn via(&self) -> Option<&'static str> {
        None
    }
}

// --- transport ------------------------------------------------------------------

pub struct Response {
    pub status: u16,
    pub content_type: String,
    pub body: Vec<u8>,
}

pub trait Transport {
    fn post(&self, url: &str, headers: &[(String, String)], content_type: &str, body: Vec<u8>) -> Result<Response, String>;
}

pub struct Http;

impl Transport for Http {
    fn post(&self, url: &str, headers: &[(String, String)], content_type: &str, body: Vec<u8>) -> Result<Response, String> {
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_global(Some(Duration::from_secs(600)))
            .user_agent(concat!("mb/", env!("CARGO_PKG_VERSION")))
            .build()
            .into();
        let mut req = agent.post(url).header("Content-Type", content_type);
        for (k, v) in headers {
            req = req.header(k.as_str(), v.as_str());
        }
        let mut resp = req.send(&body[..]).map_err(|e| format!("can't reach {}: {e}", host(url)))?;
        let status = resp.status().as_u16();
        let content_type = resp.headers().get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
        let body = resp.body_mut().with_config().limit(200 << 20).read_to_vec().map_err(|e| format!("reading the response: {e}"))?;
        Ok(Response { status, content_type, body })
    }
}

fn host(url: &str) -> &str {
    url.split('/').nth(2).unwrap_or(url)
}

/// An error message from a JSON error body, or the status.
pub fn http_error(provider: &str, r: &Response) -> String {
    let v: serde_json::Value = serde_json::from_slice(&r.body).unwrap_or_default();
    let msg = v["error"]["message"]
        .as_str()
        .or_else(|| v["error"].as_str())
        .or_else(|| v["detail"]["message"].as_str())
        .or_else(|| v["detail"][0]["msg"].as_str())
        .or_else(|| v["detail"].as_str())
        .map(str::to_string)
        .unwrap_or_else(|| String::from_utf8_lossy(&r.body).chars().take(300).collect());
    let hint = match r.status {
        401 | 403 => " (check the API key: `mb keys list`)",
        429 => " (rate limited or out of quota; wait, or check billing)",
        _ => "",
    };
    format!("{provider}: HTTP {}: {msg}{hint}", r.status)
}

/// The provider implementation for a name: with your key from the environment
/// or `mb keys` if you have one, else through Maimbrain's keys when your
/// account has access (generate/maimbrain.rs). `est` is the estimated cost of
/// the whole command, checked against what's left of a Maimbrain budget.
pub fn pick(name: &str, model: &str, est: Option<f64>) -> Result<Box<dyn Provider>, String> {
    use super::maimbrain::{self, Route};
    if !matches!(name, "mock" | "gemini" | "vertex" | "openai" | "elevenlabs") {
        return Err(format!("unknown provider {name:?} (gemini, vertex, openai, elevenlabs, maimbrain, mock)"));
    }
    let own = super::keys::lookup(name);
    match maimbrain::decide(name, Some(model), maimbrain::mode(), own.as_ref().map(|k| k.1.as_str()), &maimbrain::Live)? {
        Route::Mock => Ok(Box::new(super::mock::Mock)),
        Route::Maimbrain(s) => {
            maimbrain::check_budget(&s, est, name)?;
            eprintln!("  keys: Maimbrain's (@{}) · {}", s.username, s.summary());
            Ok(Box::new(maimbrain::Maimbrain::signed_in(name, Box::new(Http))?))
        }
        Route::Own(from) => {
            eprintln!("  keys: your {name} key (from {from})");
            let key = own.map(|k| k.0).unwrap_or_default();
            Ok(match name {
                "gemini" => Box::new(super::google::Google::gemini(key, Box::new(Http))),
                "vertex" => {
                    let location = std::env::var("GOOGLE_CLOUD_LOCATION").unwrap_or_else(|_| "global".into());
                    Box::new(super::google::Google::vertex(key, location, Box::new(Http)))
                }
                "openai" => Box::new(super::openai::OpenAi::new(key, Box::new(Http))),
                _ => Box::new(super::elevenlabs::ElevenLabs::new(key, Box::new(Http))),
            })
        }
    }
}

/// The provider to use when none was named: MB_PROVIDER, then the first with
/// a key among `order`, then (signed in, with access) the first of `order`
/// that Maimbrain's keys cover.
pub fn choose(flag: Option<&str>, style: Option<&str>, order: &[&str], media: &str) -> Result<String, String> {
    let has_key = |p: &str| super::keys::lookup(p).is_some();
    choose_with(flag, style, order, media, super::maimbrain::mode(), &has_key, &super::maimbrain::Live)
}

pub fn choose_with(
    flag: Option<&str>,
    style: Option<&str>,
    order: &[&str],
    media: &str,
    mode: super::maimbrain::Mode,
    has_key: &dyn Fn(&str) -> bool,
    server: &dyn super::maimbrain::Server,
) -> Result<String, String> {
    use super::maimbrain::{Mode, SERVER_PROVIDERS};
    if let Some(p) = flag {
        return Ok(p.to_string());
    }
    if mode == Mode::Maimbrain {
        // The style's provider if Maimbrain's keys cover it, else the first of `order` they do.
        return style
            .filter(|p| SERVER_PROVIDERS.contains(p))
            .or_else(|| order.iter().copied().find(|p| SERVER_PROVIDERS.contains(p)))
            .map(str::to_string)
            .ok_or_else(|| format!("Maimbrain's keys don't cover {media}"));
    }
    if let Some(p) = style {
        return Ok(p.to_string());
    }
    if let Ok(p) = std::env::var("MB_PROVIDER") {
        return Ok(p);
    }
    for p in order {
        if has_key(p) {
            return Ok(p.to_string());
        }
    }
    if mode == Mode::Auto
        && let Ok(Some(s)) = server.status()
        && s.allowed
        && let Some(p) = order.iter().find(|p| s.supports(p, None))
    {
        return Ok(p.to_string());
    }
    Err(format!(
        "no provider key for {media}. Set one up:\n{}\nor try the pipeline offline with --provider mock{}",
        order.iter().map(|p| format!("  {p}: {}", super::keys::help_for(p))).collect::<Vec<_>>().join("\n"),
        super::maimbrain::hint(mode, server)
    ))
}

/// Base64 (standard alphabet, padded).
pub fn b64(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

pub fn unb64(s: &str) -> Result<Vec<u8>, String> {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.decode(s.trim()).map_err(|e| format!("bad base64 in the response: {e}"))
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::cell::RefCell;

    /// A transport that records requests and answers with canned responses.
    pub struct Fake {
        pub sent: RefCell<Vec<(String, Vec<(String, String)>, serde_json::Value)>>,
        pub reply: RefCell<Vec<Response>>,
    }

    impl Fake {
        pub fn new(replies: Vec<Response>) -> Fake {
            Fake { sent: RefCell::new(Vec::new()), reply: RefCell::new(replies) }
        }
    }

    impl Transport for std::rc::Rc<Fake> {
        fn post(&self, url: &str, headers: &[(String, String)], _content_type: &str, body: Vec<u8>) -> Result<Response, String> {
            self.sent.borrow_mut().push((url.to_string(), headers.to_vec(), serde_json::from_slice(&body).unwrap_or_default()));
            let mut r = self.reply.borrow_mut();
            if r.is_empty() { Err("no canned reply".into()) } else { Ok(r.remove(0)) }
        }
    }

    pub fn json(status: u16, v: serde_json::Value) -> Response {
        Response { status, content_type: "application/json".into(), body: serde_json::to_vec(&v).unwrap() }
    }

    #[test]
    fn every_provider_has_defaults_and_prices() {
        for (p, media) in [
            ("gemini", Media::Image),
            ("gemini", Media::Music),
            ("vertex", Media::Music),
            ("openai", Media::Image),
            ("elevenlabs", Media::Sfx),
            ("mock", Media::Image),
        ] {
            let m = default_model(p, media).unwrap_or_else(|| panic!("{p} {media:?}"));
            assert!(estimate(m, "1K", 1, 30.0).is_some());
        }
        let nb = find_model("gemini", "gemini-nano-banana-2.1").unwrap();
        assert_eq!(estimate(nb, "1K", 1, 0.0), Some(0.0336));
        assert_eq!(estimate(nb, "2K", 1, 0.0), Some(0.0504));
        assert_eq!(estimate(find_model("gemini", "lyria-3-clip-preview").unwrap(), "", 0, 30.0), Some(0.04));
        let sfx = find_model("elevenlabs", "eleven_text_to_sound_v2").unwrap();
        assert!((estimate(sfx, "", 0, 2.0).unwrap() - 0.004).abs() < 1e-9);
        let oa = find_model("openai", "gpt-image-2.5-flare").unwrap();
        assert!((estimate(oa, "1K", 1, 0.0).unwrap() - 0.03168).abs() < 1e-6);
    }

    #[test]
    fn choosing_a_provider_with_and_without_maimbrains_keys() {
        use super::super::maimbrain::{Mode, Server, ServerModel, ServerProvider, Status};
        struct S(Option<Status>);
        impl Server for S {
            fn status(&self) -> Result<Option<Status>, String> {
                Ok(self.0.clone())
            }
        }
        let ok = S(Some(Status {
            allowed: true,
            username: "ada".into(),
            providers: vec![ServerProvider { name: "gemini".into(), models: vec![ServerModel { id: "gemini-nano-banana-2.1".into() }] }],
            ..Status::default()
        }));
        let order = ["gemini", "vertex", "openai"];
        let none = |_: &str| false;
        let openai = |p: &str| p == "openai";
        // Your key first, in order; then Maimbrain's keys; --own-keys never falls back.
        assert_eq!(choose_with(None, None, &order, "images", Mode::Auto, &openai, &ok).unwrap(), "openai");
        assert_eq!(choose_with(None, None, &order, "images", Mode::Auto, &none, &ok).unwrap(), "gemini");
        let e = choose_with(None, None, &order, "images", Mode::Own, &none, &ok).unwrap_err();
        assert!(e.contains("GEMINI_API_KEY") && !e.contains("Maimbrain"), "{e}");
        // --provider maimbrain: the first provider in order the server covers, over your key and the style's choice.
        assert_eq!(choose_with(None, Some("openai"), &order, "images", Mode::Maimbrain, &openai, &ok).unwrap(), "gemini");
        assert_eq!(choose_with(None, None, &["elevenlabs"], "sfx", Mode::Maimbrain, &none, &ok).unwrap(), "elevenlabs");
        assert!(choose_with(None, None, &["openai"], "images", Mode::Maimbrain, &none, &ok).is_err());
        // Signed out or without access: the setup message says how to get access.
        let e = choose_with(None, None, &order, "images", Mode::Auto, &none, &S(None)).unwrap_err();
        assert!(e.contains("ask the Maimbrain team") && e.contains("mb login"), "{e}");
        let denied = S(Some(Status { allowed: false, username: "bob".into(), ..Status::default() }));
        assert!(choose_with(None, None, &order, "images", Mode::Auto, &none, &denied).unwrap_err().contains("@bob doesn't have it"));
        // A named provider is always taken as is.
        assert_eq!(choose_with(Some("vertex"), None, &order, "images", Mode::Maimbrain, &none, &ok).unwrap(), "vertex");
    }

    #[test]
    fn errors_are_readable() {
        let r = json(403, serde_json::json!({"error": {"message": "API key not valid"}}));
        assert_eq!(http_error("gemini", &r), "gemini: HTTP 403: API key not valid (check the API key: `mb keys list`)");
        let r = json(422, serde_json::json!({"detail": [{"msg": "duration too long"}]}));
        assert!(http_error("elevenlabs", &r).contains("duration too long"));
    }
}
