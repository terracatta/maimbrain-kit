//! Talking to the Maimbrain server: `mb login`, `mb publish`, `mb whoami`.
//!
//! Credentials live in `~/.config/maimbrain/credentials.json` (mode 0600), or
//! `%APPDATA%\maimbrain\credentials.json` on Windows.
//! The server is https://maimbrain.com unless `MB_SERVER` or `--server` says
//! otherwise (e.g. http://localhost:3000 for local development).

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const DEFAULT_SERVER: &str = "https://maimbrain.com";

#[derive(Serialize, Deserialize, Clone)]
pub struct Credentials {
    pub server: String,
    pub token: String,
    pub username: String,
}

fn config_path() -> Result<PathBuf, String> {
    let base = if cfg!(windows) {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("USERPROFILE").map(|h| PathBuf::from(h).join("AppData").join("Roaming")))
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
    }
    .ok_or("can't find a home directory for credentials")?;
    Ok(base.join("maimbrain").join("credentials.json"))
}

pub fn load() -> Option<Credentials> {
    let text = std::fs::read_to_string(config_path().ok()?).ok()?;
    serde_json::from_str(&text).ok()
}

fn save(c: &Credentials) -> Result<(), String> {
    let path = config_path()?;
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(&path, serde_json::to_string_pretty(c).unwrap()).map_err(|e| e.to_string())?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

/// The server to use: --server, else MB_SERVER, else the saved login's, else maimbrain.com.
pub fn server(flag: Option<&str>) -> String {
    flag.map(str::to_string)
        .or_else(|| std::env::var("MB_SERVER").ok())
        .or_else(|| load().map(|c| c.server))
        .unwrap_or_else(|| DEFAULT_SERVER.into())
        .trim_end_matches('/')
        .to_string()
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .http_status_as_error(false)
        .timeout_global(Some(Duration::from_secs(120)))
        .user_agent(concat!("mb/", env!("CARGO_PKG_VERSION")))
        .build()
        .into()
}

/// (status, JSON body) for a request; network errors become Err.
fn json_of(resp: Result<ureq::http::Response<ureq::Body>, ureq::Error>) -> Result<(u16, Value), String> {
    let mut resp = resp.map_err(|e| format!("can't reach the server: {e}"))?;
    let status = resp.status().as_u16();
    let body = resp.body_mut().read_to_string().unwrap_or_default();
    Ok((status, serde_json::from_str(&body).unwrap_or(Value::Null)))
}

fn error_text(status: u16, body: &Value) -> String {
    body["error"].as_str().map(str::to_string).unwrap_or_else(|| format!("HTTP {status}"))
}

/// Device-code sign-in: shows a code, waits for approval on the web.
pub fn login(server_flag: Option<&str>) -> Result<(), String> {
    let server = server(server_flag);
    let host = std::env::var("HOST").or_else(|_| std::env::var("HOSTNAME")).unwrap_or_else(|_| "this computer".into());
    let (status, start) = json_of(
        agent().post(format!("{server}/api/v1/device_codes")).send_json(serde_json::json!({ "client_name": format!("mb on {host}") })),
    )?;
    if status != 201 {
        return Err(format!("couldn't start sign-in: {}", error_text(status, &start)));
    }
    let code = start["user_code"].as_str().unwrap_or("?");
    let url = start["verification_uri_complete"].as_str().unwrap_or("");
    eprintln!("To sign in, open:\n\n    {url}\n\nand confirm the code  {code}\n");
    if std::env::var_os("MB_NO_BROWSER").is_none() {
        let _ = open_browser(url);
    }
    let interval = Duration::from_secs(start["interval"].as_u64().unwrap_or(3));
    let deadline = Instant::now() + Duration::from_secs(start["expires_in"].as_u64().unwrap_or(600));
    let device_code = start["device_code"].as_str().unwrap_or_default().to_string();
    eprint!("Waiting for approval…");
    while Instant::now() < deadline {
        std::thread::sleep(interval);
        let (status, body) = json_of(
            agent().post(format!("{server}/api/v1/device_codes/token")).send_json(serde_json::json!({ "device_code": device_code })),
        )?;
        match status {
            200 => {
                let creds = Credentials {
                    server: server.clone(),
                    token: body["token"].as_str().unwrap_or_default().into(),
                    username: body["username"].as_str().unwrap_or_default().into(),
                };
                save(&creds)?;
                eprintln!("\nSigned in as @{} on {server}", creds.username);
                return Ok(());
            }
            428 => eprint!("."),
            _ => return Err(format!("\nsign-in failed: {}", error_text(status, &body))),
        }
    }
    Err("\nthe code expired; run mb login again".into())
}

fn open_browser(url: &str) -> std::io::Result<()> {
    let mut cmd = if cfg!(windows) {
        // `start` is a cmd built-in; its first quoted argument is a window title.
        let mut c = std::process::Command::new("cmd");
        c.args(["/C", "start", ""]);
        c
    } else {
        std::process::Command::new(if cfg!(target_os = "macos") { "open" } else { "xdg-open" })
    };
    cmd.arg(url).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn().map(|_| ())
}

fn credentials() -> Result<Credentials, String> {
    load().ok_or_else(|| "not signed in; run mb login".to_string())
}

pub fn whoami() -> Result<Value, String> {
    let c = credentials()?;
    let (status, body) = json_of(agent().get(format!("{}/api/v1/cli/whoami", c.server)).header("Authorization", format!("Bearer {}", c.token)).call())?;
    if status == 200 { Ok(body) } else { Err(format!("{} (try mb login)", error_text(status, &body))) }
}

/// Uploads a bundle and waits for validation; prints where it ended up.
pub fn publish(bundle: &Path) -> Result<(), String> {
    let c = credentials()?;
    let bytes = std::fs::read(bundle).map_err(|e| format!("{}: {e}", bundle.display()))?;
    eprintln!("Uploading {} ({} KB) to {}…", bundle.display(), bytes.len() / 1024, c.server);
    let (status, body) = json_of(
        agent()
            .post(format!("{}/api/v1/games/versions", c.server))
            .header("Authorization", format!("Bearer {}", c.token))
            .header("Content-Type", "application/octet-stream")
            .header("X-Filename", bundle.file_name().and_then(|n| n.to_str()).unwrap_or("game.mbx"))
            .send(&bytes[..]),
    )?;
    if status != 202 {
        return Err(format!("upload refused: {}", error_text(status, &body)));
    }
    let status_url = body["status_url"].as_str().unwrap_or_default().to_string();
    let mut v = body;
    let deadline = Instant::now() + Duration::from_secs(120);
    while v["state"] == "validating" && Instant::now() < deadline {
        std::thread::sleep(Duration::from_secs(2));
        let (s, b) = json_of(agent().get(&status_url).header("Authorization", format!("Bearer {}", c.token)).call())?;
        if s != 200 {
            return Err(format!("checking status: {}", error_text(s, &b)));
        }
        v = b;
    }
    let name = format!("{} {}", v["game_id"].as_str().unwrap_or("?"), v["version"].as_str().unwrap_or(""));
    match v["state"].as_str().unwrap_or("") {
        "pending" => {
            eprintln!("✓ {name} passed validation and is waiting for review.");
            eprintln!("  Track it at {}/create", c.server);
            Ok(())
        }
        "invalid" => {
            let errors: Vec<&str> = v["errors"].as_array().map(|a| a.iter().filter_map(|e| e.as_str()).collect()).unwrap_or_default();
            Err(format!("the server rejected the bundle:\n  - {}", errors.join("\n  - ")))
        }
        "validating" => {
            eprintln!("Uploaded; validation is still running. Check {}/create", c.server);
            Ok(())
        }
        other => {
            eprintln!("{name}: {other}");
            Ok(())
        }
    }
}
