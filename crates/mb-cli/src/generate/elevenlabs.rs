//! ElevenLabs (optional provider): sound effects (`/v1/sound-generation`,
//! 0.5–30 s, optionally made to loop) and music (`/v1/music`, an exact
//! length, instrumental). Responses are the audio itself.
//! https://elevenlabs.io/docs/api-reference/text-to-sound-effects/convert,
//! https://elevenlabs.io/docs/api-reference/music/compose

use serde_json::json;

use super::audio::Kind;
use super::provider::{AudioRequest, ImageRequest, Output, Provider, Response, Transport, http_error};

pub struct ElevenLabs {
    key: String,
    http: Box<dyn Transport>,
}

impl ElevenLabs {
    pub fn new(key: String, http: Box<dyn Transport>) -> ElevenLabs {
        ElevenLabs { key, http }
    }

    fn post(&self, path: &str, format: &str, body: &serde_json::Value) -> Result<Response, String> {
        let url = format!("https://api.elevenlabs.io{path}?output_format={format}");
        self.http.post(&url, &[("xi-api-key".to_string(), self.key.clone())], "application/json", serde_json::to_vec(body).unwrap())
    }
}

fn mime_for(format: &str, content_type: &str) -> String {
    if let Some(rate) = format.strip_prefix("pcm_") {
        format!("audio/pcm;rate={rate}")
    } else if content_type.starts_with("audio/") {
        content_type.split(';').next().unwrap_or("audio/mpeg").to_string()
    } else {
        "audio/mpeg".into()
    }
}

impl Provider for ElevenLabs {
    fn name(&self) -> &'static str {
        "elevenlabs"
    }

    fn image(&self, _req: &ImageRequest) -> Result<Output, String> {
        Err("elevenlabs makes sound, not images; use --provider gemini or openai".into())
    }

    fn audio(&self, req: &AudioRequest) -> Result<Output, String> {
        let (path, body) = match req.kind {
            Kind::Sfx => {
                let mut b = json!({ "text": req.prompt, "model_id": req.model, "prompt_influence": 0.5, "loop": req.looping });
                if let Some(s) = req.seconds {
                    b["duration_seconds"] = json!(s.clamp(0.5, 30.0));
                }
                ("/v1/sound-generation", b)
            }
            Kind::Music => {
                let mut b = json!({ "prompt": req.prompt, "model_id": req.model, "force_instrumental": true });
                if let Some(s) = req.seconds {
                    b["music_length_ms"] = json!(((s * 1000.0) as u32).clamp(3000, 600_000));
                }
                ("/v1/music", b)
            }
        };
        // Raw PCM where the plan allows it (no MP3 decoding, no encoder delay), else MP3.
        let mut format = if req.kind == Kind::Sfx { "pcm_24000" } else { "mp3_44100_128" };
        let mut r = self.post(path, format, &body)?;
        if req.kind == Kind::Sfx && (r.status == 403 || r.status == 422 || r.status == 400) && String::from_utf8_lossy(&r.body).contains("output_format") {
            format = "mp3_44100_128";
            r = self.post(path, format, &body)?;
        }
        if r.status != 200 {
            return Err(http_error("elevenlabs", &r));
        }
        let seconds = req.seconds.unwrap_or(if req.kind == Kind::Sfx { 2.0 } else { 30.0 });
        let per_min = if req.kind == Kind::Sfx { 0.12 } else { 0.15 };
        Ok(Output { mime: mime_for(format, &r.content_type), bytes: r.body, text: None, cost_usd: Some(per_min * seconds as f64 / 60.0) })
    }
}

#[cfg(test)]
mod tests {
    use super::super::provider::tests::{Fake, json as reply};
    use super::*;
    use std::rc::Rc;

    fn audio(status: u16, body: &[u8]) -> Response {
        Response { status, content_type: "audio/mpeg".into(), body: body.to_vec() }
    }

    #[test]
    fn sound_effect_request_with_mp3_fallback() {
        let fake =
            Rc::new(Fake::new(vec![reply(403, json!({ "detail": { "message": "output_format pcm_24000 requires a higher tier" } })), audio(200, b"ID3...")]));
        let e = ElevenLabs::new("xi".into(), Box::new(fake.clone()));
        let req = AudioRequest {
            model: "eleven_text_to_sound_v2".into(),
            prompt: "a glassy chime".into(),
            kind: Kind::Sfx,
            seconds: Some(0.2),
            looping: false,
            seed: None,
            negative: None,
        };
        let out = e.audio(&req).unwrap();
        assert_eq!(out.mime, "audio/mpeg");
        let sent = fake.sent.borrow();
        assert!(sent[0].0.ends_with("/v1/sound-generation?output_format=pcm_24000"));
        assert!(sent[1].0.ends_with("output_format=mp3_44100_128"));
        assert_eq!(sent[0].1[0], ("xi-api-key".to_string(), "xi".to_string()));
        assert_eq!(sent[0].2["duration_seconds"], 0.5, "clamped to the API's minimum");
        assert_eq!(sent[0].2["text"], "a glassy chime");
    }

    #[test]
    fn pcm_mime_and_music() {
        let fake = Rc::new(Fake::new(vec![audio(200, &[0, 0]), audio(200, b"ID3")]));
        let e = ElevenLabs::new("xi".into(), Box::new(fake.clone()));
        let mut req = AudioRequest {
            model: "eleven_text_to_sound_v2".into(),
            prompt: "zap".into(),
            kind: Kind::Sfx,
            seconds: None,
            looping: true,
            seed: None,
            negative: None,
        };
        assert_eq!(e.audio(&req).unwrap().mime, "audio/pcm;rate=24000");
        req.kind = Kind::Music;
        req.model = "music_v2_5".into();
        req.seconds = Some(16.0);
        e.audio(&req).unwrap();
        let body = &fake.sent.borrow()[1].2;
        assert_eq!(body["music_length_ms"], 16000);
        assert_eq!(body["force_instrumental"], true);
    }
}
