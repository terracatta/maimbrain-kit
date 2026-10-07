//! `mb new <name>`: a new game crate from the maimbrain-game skill's
//! template/ (2D) or template-3d/ (`--3d`). Works signed out: the id and
//! creator come from `--id`/`--creator`, else the signed-in account, else a
//! placeholder that builds and previews but can't be published.

use std::path::Path;

macro_rules! template {
    ($dir:literal, $path:literal) => {
        include_str!(concat!(env!($dir), "/", $path))
    };
}

struct Template {
    /// (path in the crate, contents); `NAME` is replaced with the crate name.
    files: &'static [(&'static str, &'static str)],
    icon: &'static [u8],
}

const TEMPLATE_2D: Template = Template {
    files: &[
        ("Cargo.toml", template!("MB_TEMPLATE_DIR", "Cargo.toml")),
        ("manifest.toml", template!("MB_TEMPLATE_DIR", "manifest.toml")),
        ("src/lib.rs", template!("MB_TEMPLATE_DIR", "src/lib.rs")),
        ("src/sim.rs", template!("MB_TEMPLATE_DIR", "src/sim.rs")),
    ],
    icon: include_bytes!(concat!(env!("MB_TEMPLATE_DIR"), "/icon.png")),
};

const TEMPLATE_3D: Template = Template {
    files: &[
        ("Cargo.toml", template!("MB_TEMPLATE_3D_DIR", "Cargo.toml")),
        ("manifest.toml", template!("MB_TEMPLATE_3D_DIR", "manifest.toml")),
        ("src/lib.rs", template!("MB_TEMPLATE_3D_DIR", "src/lib.rs")),
        ("src/sim.rs", template!("MB_TEMPLATE_3D_DIR", "src/sim.rs")),
        ("src/scene.rs", template!("MB_TEMPLATE_3D_DIR", "src/scene.rs")),
    ],
    icon: include_bytes!(concat!(env!("MB_TEMPLATE_3D_DIR"), "/icon.png")),
};

/// Where the SDK comes from outside this monorepo.
const SDK_GIT: &str = "https://github.com/terracatta/maimbrain-kit";
/// Used when there's no `--id` and nobody is signed in.
const PLACEHOLDER_NAMESPACE: &str = "com.maimbrain.yourname";
const PLACEHOLDER_CREATOR: &str = "yourname";

pub struct Options<'a> {
    pub three_d: bool,
    /// Explicit bundle id (reverse-DNS), overriding the account's namespace.
    pub id: Option<&'a str>,
    /// Explicit creator handle (with or without the leading @).
    pub creator: Option<&'a str>,
    /// (username, id namespace) from the server, if signed in.
    pub who: Option<(&'a str, &'a str)>,
}

pub fn new_game(dir: &Path, opts: &Options) -> Result<(), String> {
    let name = dir.file_name().and_then(|n| n.to_str()).ok_or("give the game a directory name, e.g. mb new frogger")?;
    if !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') || name.is_empty() {
        return Err(format!("{name:?}: use lowercase letters, digits and _ (it becomes the crate name and the id's last part)"));
    }
    if dir.exists() {
        return Err(format!("{} already exists", dir.display()));
    }
    let namespace = opts.who.map_or(PLACEHOLDER_NAMESPACE, |w| w.1);
    let id = opts.id.map_or_else(|| format!("{namespace}.{}", name.replace('_', "-")), str::to_string);
    let creator = opts.creator.map(|c| c.trim_start_matches('@')).or(opts.who.map(|w| w.0)).unwrap_or(PLACEHOLDER_CREATOR);
    let template = if opts.three_d { &TEMPLATE_3D } else { &TEMPLATE_2D };
    // Inside the Maimbrain monorepo (games/<name>) the crate joins the workspace; elsewhere it's standalone.
    let in_monorepo = dir.parent().map(|p| p.join("../sdk/maimbrain/Cargo.toml").exists()).unwrap_or(false);

    let mut files = Vec::new();
    for &(path, text) in template.files {
        let text = text.replace("NAME", name);
        let text = match path {
            "Cargo.toml" if !in_monorepo => standalone_cargo(&text),
            "manifest.toml" => manifest(&text, &id, creator, &title_case(name)),
            _ => text,
        };
        files.push((path, text));
    }
    // Check the manifest before writing anything (a bad --id fails here).
    let m = files.iter().find(|f| f.0 == "manifest.toml").map(|f| f.1.as_str()).unwrap_or_default();
    let parsed = mb_format::Manifest::parse(m).map_err(|e| format!("manifest.toml: {e}"))?;
    let mut errors = Vec::new();
    parsed.check(&mut errors);
    if !errors.is_empty() {
        return Err(errors.join("; "));
    }

    std::fs::create_dir_all(dir.join("src")).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(dir.join("assets")).map_err(|e| e.to_string())?;
    let write = |p: &str, c: &[u8]| std::fs::write(dir.join(p), c).map_err(|e| format!("{p}: {e}"));
    for (path, text) in &files {
        write(path, text.as_bytes())?;
    }
    write("icon.png", template.icon)?;
    eprintln!("Created {} ({}, id {id}, creator @{creator})", dir.display(), if opts.three_d { "3D" } else { "2D" });
    if opts.who.is_none() && (opts.id.is_none() || opts.creator.is_none()) {
        eprintln!(
            "Not signed in, so manifest.toml has a placeholder id/creator. It builds, validates and previews as is; \
             before `mb publish`, run `mb login` and set id and creator to your namespace (`mb whoami`), \
             or pass --id and --creator to mb new."
        );
    }
    eprintln!("Next: mb build {0}   then mb serve {0}", dir.display());
    Ok(())
}

fn standalone_cargo(text: &str) -> String {
    text.replace("edition.workspace = true", "edition = \"2024\"")
        .replace("version.workspace = true", "version = \"0.1.0\"")
        .replace("publish.workspace = true", "publish = false")
        .replace("maimbrain = { path = \"../../sdk/maimbrain\" }", &format!("maimbrain = {{ git = \"{SDK_GIT}\" }}"))
        + "\n# Standalone: not part of any parent workspace.\n[workspace]\n\n[profile.release]\nopt-level = \"s\"\nlto = true\ncodegen-units = 1\npanic = \"abort\"\n"
}

/// Sets the manifest's id, creator and name lines.
fn manifest(text: &str, id: &str, creator: &str, title: &str) -> String {
    text.lines()
        .map(|l| {
            if l.starts_with("id = ") {
                format!("id = \"{id}\"")
            } else if l.starts_with("creator = ") {
                format!("creator = \"@{creator}\"")
            } else if l.starts_with("name = ") {
                format!("name = \"{title}\"")
            } else {
                l.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

fn title_case(s: &str) -> String {
    s.split('_')
        .filter(|w| !w.is_empty())
        .map(|w| w[..1].to_uppercase() + &w[1..])
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("mb-new-test-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    fn read_manifest(dir: &Path) -> mb_format::Manifest {
        mb_format::Manifest::parse(&std::fs::read_to_string(dir.join("manifest.toml")).unwrap()).unwrap()
    }

    #[test]
    fn signed_out_uses_a_placeholder_that_validates() {
        let root = scratch("out");
        let dir = root.join("frog_hop");
        new_game(&dir, &Options { three_d: false, id: None, creator: None, who: None }).unwrap();
        let m = read_manifest(&dir);
        assert_eq!(m.id, "com.maimbrain.yourname.frog-hop");
        assert_eq!(m.creator, "@yourname");
        assert_eq!(m.name, "Frog Hop");
        let cargo = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap();
        assert!(cargo.contains("name = \"frog_hop\"") && cargo.contains("[workspace]"), "standalone outside the monorepo");
        assert!(dir.join("icon.png").is_file() && dir.join("src/sim.rs").is_file());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn three_d_with_an_explicit_id_and_creator() {
        let root = scratch("3d");
        let dir = root.join("sky");
        new_game(&dir, &Options { three_d: true, id: Some("dev.example.sky"), creator: Some("@ada"), who: None }).unwrap();
        let m = read_manifest(&dir);
        assert_eq!((m.id.as_str(), m.creator.as_str(), m.name.as_str()), ("dev.example.sky", "@ada", "Sky"));
        assert!(m.has_stdlib("mb3d") && m.has_stdlib("mb2d"));
        assert!(dir.join("src/scene.rs").is_file());
        assert!(!std::fs::read_to_string(dir.join("src/lib.rs")).unwrap().contains("NAME"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn signed_in_uses_the_namespace_and_bad_ids_fail_before_writing() {
        let root = scratch("in");
        let dir = root.join("pop");
        new_game(&dir, &Options { three_d: false, id: None, creator: None, who: Some(("ada_l", "com.maimbrain.ada-l")) }).unwrap();
        let m = read_manifest(&dir);
        assert_eq!((m.id.as_str(), m.creator.as_str()), ("com.maimbrain.ada-l.pop", "@ada_l"));
        let bad = root.join("bad");
        assert!(new_game(&bad, &Options { three_d: false, id: Some("Not An Id"), creator: None, who: None }).is_err());
        assert!(!bad.exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
