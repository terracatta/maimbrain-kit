//! `mb serve`: preview a game in a desktop browser (SPEC §9), using the web
//! runtime embedded in this binary. Reloading the page rebuilds the game
//! first if any of its files changed, so the loop is: save, reload.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

macro_rules! runtime_file {
    ($f:literal) => {
        ($f, include_bytes!(concat!(env!("OUT_DIR"), "/runtime/", $f)) as &[u8])
    };
}

const RUNTIME: [(&str, &[u8]); 5] = [
    runtime_file!("index.html"),
    runtime_file!("runtime.js"),
    runtime_file!("probe-worker.js"),
    runtime_file!("mb_host_bg.wasm"),
    runtime_file!("silence.wav"),
];

pub fn serve(
    path: &Path,
    port: u16,
    build: impl Fn(&Path) -> Result<PathBuf, String>,
    stage: impl Fn(&Path, &Path, &Path) -> Result<(), String>,
) -> Result<(), String> {
    if env!("MB_RUNTIME_EMBEDDED").is_empty() {
        return Err(format!(
            "this mb was built without the web runtime ({} was missing). In the Maimbrain repo, run `just runtime`, \
             then run mb again through cargo (`cargo run -p mb-cli -- serve …`), which embeds it automatically. \
             Otherwise install a release build: curl -fsSL https://maimbrain.com/install.sh | sh",
            env!("MB_RUNTIME_DIST")
        ));
    }
    let work = std::env::temp_dir().join(format!("mb-serve-{}", std::process::id()));
    let runtime_dir = work.join("runtime-src");
    std::fs::create_dir_all(&runtime_dir).map_err(|e| e.to_string())?;
    for (name, bytes) in RUNTIME {
        std::fs::write(runtime_dir.join(name), bytes).map_err(|e| e.to_string())?;
    }
    let site = work.join("site");
    let is_dir = path.is_dir();
    let rebuild = |site: &Path| -> Result<(), String> {
        let bundle = if is_dir { build(path)? } else { path.to_path_buf() };
        stage(&bundle, site, &runtime_dir)
    };
    rebuild(&site)?;
    let mut built_at = SystemTime::now();

    let server = tiny_http::Server::http(("127.0.0.1", port)).map_err(|e| format!("can't listen on port {port}: {e}"))?;
    eprintln!("\nPreview: http://127.0.0.1:{port}/runtime/index.html?overlays");
    eprintln!("Reload the page after saving to rebuild. Ctrl-C to stop.\n");
    for req in server.incoming_requests() {
        let url = req.url().split('?').next().unwrap_or("/").to_string();
        if url == "/" || url == "/runtime/index.html" {
            if is_dir && newest_change(path) > built_at {
                eprintln!("change detected, rebuilding…");
                match rebuild(&site) {
                    Ok(()) => built_at = SystemTime::now(),
                    Err(e) => eprintln!("build failed: {e}"),
                }
            }
            if url == "/" {
                let r = tiny_http::Response::empty(302).with_header(header("Location", "/runtime/index.html?overlays"));
                let _ = req.respond(r);
                continue;
            }
        }
        let rel = url.trim_start_matches('/');
        let file = site.join(rel);
        if rel.split('/').any(|c| c == "..") || !file.is_file() {
            let _ = req.respond(tiny_http::Response::from_string("not found").with_status_code(404));
            continue;
        }
        let bytes = std::fs::read(&file).unwrap_or_default();
        let r = tiny_http::Response::from_data(bytes)
            .with_header(header("Content-Type", content_type(&file)))
            .with_header(header("Cache-Control", "no-store"));
        let _ = req.respond(r);
    }
    Ok(())
}

fn header(k: &str, v: &str) -> tiny_http::Header {
    tiny_http::Header::from_bytes(k.as_bytes(), v.as_bytes()).unwrap()
}

fn content_type(p: &Path) -> &'static str {
    match p.extension().and_then(|e| e.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript",
        "wasm" => "application/wasm",
        "json" => "application/json",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "glb" => "model/gltf-binary",
        "ogg" => "audio/ogg",
        "wav" => "audio/wav",
        _ => "application/octet-stream",
    }
}

/// The latest modification time of the game's sources.
fn newest_change(dir: &Path) -> SystemTime {
    let mut newest = SystemTime::UNIX_EPOCH;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else { continue };
        for e in entries.flatten() {
            let p = e.path();
            let name = e.file_name();
            if name.to_string_lossy().starts_with('.') || name == "target" {
                continue;
            }
            if p.is_dir() {
                stack.push(p);
            } else if let Ok(m) = e.metadata().and_then(|m| m.modified()) {
                newest = newest.max(m);
            }
        }
    }
    newest
}
