//! `mb new <name>`: a new game crate from the maimbrain-game skill's template.

use std::path::Path;

macro_rules! template {
    ($path:literal) => {
        include_str!(concat!(env!("MB_TEMPLATE_DIR"), "/", $path))
    };
}

const CARGO: &str = template!("Cargo.toml");
const MANIFEST: &str = template!("manifest.toml");
const LIB: &str = template!("src/lib.rs");
const SIM: &str = template!("src/sim.rs");
const ICON: &[u8] = include_bytes!(concat!(env!("MB_TEMPLATE_DIR"), "/icon.png"));

/// Where the SDK comes from outside this monorepo.
const SDK_GIT: &str = "https://github.com/terracatta/maimbrain-kit";

pub fn new_game(dir: &Path, username: Option<&str>) -> Result<(), String> {
    let name = dir.file_name().and_then(|n| n.to_str()).ok_or("give the game a directory name, e.g. mb new frogger")?;
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') || name.is_empty() {
        return Err(format!("{name:?}: use lowercase letters, digits and _ (it becomes the crate name and the id's last part)"));
    }
    if dir.exists() {
        return Err(format!("{} already exists", dir.display()));
    }
    let user = username.unwrap_or("YOURNAME");
    // Inside the Maimbrain monorepo (games/<name>) the crate joins the workspace; elsewhere it's standalone.
    let in_monorepo = dir.parent().map(|p| p.join("../sdk/maimbrain/Cargo.toml").exists()).unwrap_or(false);
    let cargo = if in_monorepo {
        CARGO.replace("NAME", name)
    } else {
        CARGO
            .replace("NAME", name)
            .replace("edition.workspace = true", "edition = \"2024\"")
            .replace("version.workspace = true", "version = \"0.1.0\"")
            .replace("publish.workspace = true", "publish = false")
            .replace("maimbrain = { path = \"../../sdk/maimbrain\" }", &format!("maimbrain = {{ git = \"{SDK_GIT}\" }}"))
            + "\n# Standalone: not part of any parent workspace.\n[workspace]\n\n[profile.release]\nopt-level = \"s\"\nlto = true\ncodegen-units = 1\npanic = \"abort\"\n"
    };
    let manifest = MANIFEST
        .replace("id = \"dev.maimbrain.NAME\"", &format!("id = \"com.maimbrain.{user}.{name}\""))
        .replace("creator = \"@maimbrain\"", &format!("creator = \"@{user}\""))
        .replace("name = \"Bubble\"", &format!("name = \"{}\"", title_case(name)));
    std::fs::create_dir_all(dir.join("src")).map_err(|e| e.to_string())?;
    let write = |p: &str, c: &[u8]| std::fs::write(dir.join(p), c).map_err(|e| format!("{p}: {e}"));
    write("Cargo.toml", cargo.as_bytes())?;
    write("manifest.toml", manifest.as_bytes())?;
    write("src/lib.rs", LIB.replace("NAME", name).as_bytes())?;
    write("src/sim.rs", SIM.replace("NAME", name).as_bytes())?;
    write("icon.png", ICON)?;
    std::fs::create_dir_all(dir.join("assets")).map_err(|e| e.to_string())?;
    eprintln!("Created {} (id com.maimbrain.{user}.{name})", dir.display());
    if username.is_none() {
        eprintln!("Not signed in: run mb login, then replace YOURNAME in manifest.toml with your username.");
    }
    eprintln!("Next: mb build {}   (then mb publish {})", dir.display(), dir.display());
    Ok(())
}

fn title_case(s: &str) -> String {
    s.split('_')
        .filter(|w| !w.is_empty())
        .map(|w| w[..1].to_uppercase() + &w[1..])
        .collect::<Vec<_>>()
        .join(" ")
}
