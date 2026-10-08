//! OpenAI's GPT Image models (optional provider): `/v1/images/generations`,
//! or `/v1/images/edits` when there are reference images. They make real
//! transparent backgrounds (`background: "transparent"`), so sprites skip
//! keying, but take no seed. Billed by tokens; the response's `usage` gives
//! the actual cost. https://developers.openai.com/api/docs/guides/image-generation

use serde_json::{Value, json};

use super::provider::{AudioRequest, ImageRequest, Output, Provider, Transport, http_error, unb64};

pub struct OpenAi {
    key: String,
    http: Box<dyn Transport>,
}

impl OpenAi {
    pub fn new(key: String, http: Box<dyn Transport>) -> OpenAi {
        OpenAi { key, http }
    }
}

/// WIDTHxHEIGHT for an aspect ratio at a resolution tier: multiples of 16,
/// about the tier's pixel count, within the API's limits (≥ 655,360 px,
/// edges ≤ 3840, ratio between 1:3 and 3:1).
pub fn size_for(aspect: &str, resolution: &str) -> String {
    let (aw, ah) = aspect.split_once(':').and_then(|(a, b)| Some((a.parse::<f64>().ok()?, b.parse::<f64>().ok()?))).unwrap_or((1.0, 1.0));
    let ratio = (aw / ah).clamp(1.0 / 3.0, 3.0);
    let pixels: f64 = match resolution {
        "2K" => 2048.0 * 2048.0,
        "4K" => 8_294_400.0,
        _ => 1024.0 * 1024.0,
    };
    let r16 = |v: f64| ((v / 16.0).round() * 16.0).clamp(16.0, 3840.0) as u32;
    let mut w = r16((pixels * ratio).sqrt());
    let mut h = r16((pixels / ratio).sqrt());
    while (w as u64 * h as u64) < 655_360 {
        w += 16;
        h = r16(w as f64 / ratio);
    }
    while (w as u64 * h as u64) > 8_294_400 {
        w -= 16;
        h = r16(w as f64 / ratio);
    }
    format!("{w}x{h}")
}

fn cost(usage: &Value) -> Option<f64> {
    let out = usage["output_tokens"].as_f64()?;
    let img_in = usage["input_tokens_details"]["image_tokens"].as_f64().unwrap_or(0.0);
    let txt_in = usage["input_tokens_details"]["text_tokens"].as_f64().unwrap_or_else(|| usage["input_tokens"].as_f64().unwrap_or(0.0) - img_in);
    Some((out * 30.0 + img_in * 8.0 + txt_in * 5.0) / 1e6)
}

fn multipart(fields: &[(&str, String)], files: &[(String, Vec<u8>, String)]) -> (String, Vec<u8>) {
    let boundary = format!("mb-boundary-{:016x}", (std::process::id() as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15));
    let mut b = Vec::new();
    for (k, v) in fields {
        b.extend_from_slice(format!("--{boundary}\r\nContent-Disposition: form-data; name=\"{k}\"\r\n\r\n{v}\r\n").as_bytes());
    }
    for (i, (mime, data, ext)) in files.iter().map(|(m, d, e)| (m, d, e)).enumerate() {
        b.extend_from_slice(
            format!("--{boundary}\r\nContent-Disposition: form-data; name=\"image[]\"; filename=\"ref{i}.{ext}\"\r\nContent-Type: {mime}\r\n\r\n").as_bytes(),
        );
        b.extend_from_slice(data);
        b.extend_from_slice(b"\r\n");
    }
    b.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    (format!("multipart/form-data; boundary={boundary}"), b)
}

impl Provider for OpenAi {
    fn name(&self) -> &'static str {
        "openai"
    }

    fn image(&self, req: &ImageRequest) -> Result<Output, String> {
        let quality = ["low", "medium", "high"][req.quality.min(2)];
        let size = size_for(&req.aspect, &req.resolution);
        let background = if req.transparent { "transparent" } else { "opaque" };
        let auth = vec![("Authorization".to_string(), format!("Bearer {}", self.key))];
        let r = if req.refs.is_empty() {
            let body = json!({
                "model": req.model, "prompt": req.prompt, "size": size, "quality": quality,
                "background": background, "output_format": "png", "n": 1,
            });
            self.http.post("https://api.openai.com/v1/images/generations", &auth, "application/json", serde_json::to_vec(&body).unwrap())?
        } else {
            let fields = [
                ("model", req.model.clone()),
                ("prompt", req.prompt.clone()),
                ("size", size),
                ("quality", quality.to_string()),
                ("background", background.to_string()),
                ("output_format", "png".to_string()),
            ];
            let files: Vec<(String, Vec<u8>, String)> =
                req.refs.iter().map(|r| (r.mime.clone(), r.bytes.clone(), if r.mime.contains("jpeg") { "jpg" } else { "png" }.to_string())).collect();
            let (ct, body) = multipart(&fields, &files);
            self.http.post("https://api.openai.com/v1/images/edits", &auth, &ct, body)?
        };
        if r.status != 200 {
            return Err(http_error("openai", &r));
        }
        let v: Value = serde_json::from_slice(&r.body).map_err(|e| format!("openai: unreadable response: {e}"))?;
        let data = v["data"][0]["b64_json"].as_str().ok_or("openai: no image in the response")?;
        Ok(Output {
            bytes: unb64(data)?,
            mime: "image/png".into(),
            text: v["data"][0]["revised_prompt"].as_str().map(str::to_string),
            cost_usd: cost(&v["usage"]),
        })
    }

    fn audio(&self, _req: &AudioRequest) -> Result<Output, String> {
        Err("openai has no music or sound-effect model in mb; use --provider gemini (music) or elevenlabs (sfx)".into())
    }
}

#[cfg(test)]
mod tests {
    use super::super::provider::tests::{Fake, json as reply};
    use super::super::provider::{MockHint, RefImage, b64};
    use super::*;
    use std::rc::Rc;

    #[test]
    fn sizes_follow_the_rules() {
        assert_eq!(size_for("1:1", "1K"), "1024x1024");
        for (a, r) in [("9:16", "1K"), ("16:9", "2K"), ("21:9", "1K"), ("1:1", "4K")] {
            let s = size_for(a, r);
            let (w, h) = s.split_once('x').unwrap();
            let (w, h): (u32, u32) = (w.parse().unwrap(), h.parse().unwrap());
            assert!(w % 16 == 0 && h % 16 == 0 && w * h >= 655_360 && w * h <= 8_294_400 && w <= 3840 && h <= 3840, "{a} {r} {s}");
        }
    }

    #[test]
    fn generation_and_cost() {
        let fake = Rc::new(Fake::new(vec![reply(
            200,
            json!({ "data": [ { "b64_json": b64(b"png") } ], "usage": { "input_tokens": 50, "output_tokens": 1000, "input_tokens_details": { "text_tokens": 50, "image_tokens": 0 } } }),
        )]));
        let o = OpenAi::new("sk-test".into(), Box::new(fake.clone()));
        let req = ImageRequest {
            model: "gpt-image-2.5-flare".into(),
            prompt: "a moth".into(),
            refs: vec![],
            aspect: "1:1".into(),
            resolution: "1K".into(),
            transparent: true,
            seed: Some(1),
            quality: 1,
            hint: MockHint::default(),
        };
        let out = o.image(&req).unwrap();
        assert_eq!(out.bytes, b"png");
        assert!((out.cost_usd.unwrap() - 0.03025).abs() < 1e-9);
        let (url, headers, body) = &fake.sent.borrow()[0];
        assert_eq!(url, "https://api.openai.com/v1/images/generations");
        assert_eq!(headers[0].1, "Bearer sk-test");
        assert_eq!(body["background"], "transparent");
        assert_eq!(body["quality"], "medium");
        assert!(body.get("seed").is_none());
    }

    #[test]
    fn references_use_multipart_edits() {
        let fake = Rc::new(Fake::new(vec![reply(200, json!({ "data": [ { "b64_json": b64(b"x") } ] }))]));
        let o = OpenAi::new("sk".into(), Box::new(fake.clone()));
        let req = ImageRequest {
            model: "gpt-image-2.5-flare".into(),
            prompt: "restyle".into(),
            refs: vec![RefImage { bytes: b"\xff\xd8jpeg".to_vec(), mime: "image/jpeg".into() }],
            aspect: "1:1".into(),
            resolution: "1K".into(),
            transparent: false,
            seed: None,
            quality: 0,
            hint: MockHint::default(),
        };
        o.image(&req).unwrap();
        assert_eq!(fake.sent.borrow()[0].0, "https://api.openai.com/v1/images/edits");
    }
}
