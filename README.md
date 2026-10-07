# Maimbrain creator kit

Make games for [Maimbrain](https://maimbrain.com): tiny games people swipe through on their phones. Games are Rust → WebAssembly; you describe one, and your agent builds it with the `maimbrain-game` skill.

## Get started

macOS or Linux:

```sh
curl -fsSL https://maimbrain.com/install.sh | sh   # the mb tool
mb doctor                                          # checks Rust, the wasm32 target, Python and the sound tools
mb login                                           # sign in to maimbrain.com
```

Windows (PowerShell):

```powershell
irm https://maimbrain.com/install.ps1 | iex        # installs mb.exe and adds it to your PATH
mb doctor
mb login
```

On Windows, install Rust with [rustup](https://rustup.rs) (it sets up the Visual Studio C++ build tools), then `rustup target add wasm32-unknown-unknown`. The skill's art and sound scripts need Python 3 (`winget install Python.Python.3.12`) and `oggenc` and `oggdec` (Windows builds are at [RareWares](https://www.rarewares.org/ogg-oggenc.php); put them on your PATH); use `python` or `py` where the skill says `python3`. WSL works too: follow the Linux steps inside it.

In Claude Code:

```
/plugin marketplace add terracatta/maimbrain-kit
/plugin install maimbrain@maimbrain-kit
```

Then ask: *"Use the maimbrain-game skill to make a game: …"*. Your agent will run:

```sh
mb new mygame        # a game crate from the template, with your id (--3d for a 3D one; works signed out too)
mb serve mygame      # preview at http://127.0.0.1:8765 (reload to rebuild)
mb publish mygame    # upload for review; track it at maimbrain.com/create
```

## What's here

| Path | What |
|---|---|
| `skills/maimbrain-game/` | The skill: workflow, engagement guide, 2D `template/` and 3D `template-3d/`, `pixel.py`, `sfx.py`, `check_audio.py`, `make_glb.py` |
| `docs/SPEC.md` | The game spec (manifest, host API, limits, determinism) |
| `docs/MB3D.md` | The 3D engine: PBR, sky lighting, shadows, post effects, particles, trails, glTF |
| `sdk/maimbrain/` | The Rust SDK (games depend on it via git) |
| `crates/mb-cli/` | The `mb` tool |
| `crates/mb-format/` | The bundle format and validator (the same one the server runs) |

Build `mb` from source: `cargo install --git https://github.com/terracatta/maimbrain-kit mb-cli`.

This repository is generated from the Maimbrain monorepo; open issues here.

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option. Games you make with it are yours.
