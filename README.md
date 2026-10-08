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
mb serve --watch mygame  # preview at http://127.0.0.1:8765; rebuilds and updates the page on every save
mb publish mygame    # upload a private draft: play it in the Maimbrain app (Account → My games)
mb submit mygame     # when it's right, send it for review; track it at maimbrain.com/create
```

`mb publish` uploads a draft only you can see; publish again to replace it while you test. Once a moderator approves what you submit, it's in everyone's feed. `mb publish --submit` skips the draft and goes straight to review.

### Generated art and sound (optional)

With an API key, `mb` can generate a game's art and music in a consistent style and post-process it to the platform's rules:

```sh
mb keys set gemini                          # a Gemini API key with billing (images and music)
mb art init mygame                          # mygame/art/style.toml: style words, palette, references
mb art sprite mygame hero "a frog in a tiny knight's helmet"
mb art background mygame sky "a misty pond at dawn"
mb music mygame music "bouncy marimba and claps" --bars 8
mb sfx mygame croak "a short wet croak"     # sound effects need an ElevenLabs key (free plan available)
mb models                                   # models, prices, which keys are set
```

Each call shows its estimated cost (a few cents) and every asset gets a `.gen.json` provenance sidecar. `--provider mock` tries everything offline. See [docs/GENERATE.md](docs/GENERATE.md) for providers, keys, the style guide and the pipeline.

No key of your own? If the Maimbrain team has given your account access to **Maimbrain's keys**, `mb login` is all you need: without a key, the commands above go through the Maimbrain server, within a monthly budget (`mb models` shows what's left). Your own key always comes first; `--own-keys` never uses Maimbrain's. See "Maimbrain's keys" in docs/GENERATE.md.

## What's here

| Path | What |
|---|---|
| `skills/maimbrain-game/` | The skill: workflow, engagement guide, art and sound guide, 2D `template/` and 3D `template-3d/`, `pixel.py`, `sfx.py`, `check_audio.py`, `make_glb.py`, `make_rigged_glb.py` (a rigged, animated character) |
| `docs/SPEC.md` | The game spec (manifest, host API, limits, determinism) |
| `docs/MB3D.md` | The 3D engine: PBR, sky lighting, shadows, post effects, particles, trails, glTF; mb3d 2: animated characters, instancing, fog, toon and outlines, 3D text, depth of field |
| `docs/GENERATE.md` | Generated art and sound: `mb art`, `mb music`, `mb sfx`, providers and keys, the style guide |
| `docs/UI.md` | The UI and juice kit: themes, buttons, icons, text, motion, confetti, title and results cards |
| `examples/gallery/` | Every kit widget and effect on nine pages: `mb serve examples/gallery` |
| `docs/PHYSICS.md` | Deterministic 2D/3D rigid-body physics in the SDK (`features = ["physics2d"]` / `["physics3d"]`) |
| `sdk/maimbrain/` | The Rust SDK (games depend on it via git) |
| `crates/mb-cli/` | The `mb` tool |
| `crates/mb-format/` | The bundle format and validator (the same one the server runs) |

Build `mb` from source: `cargo install --git https://github.com/terracatta/maimbrain-kit mb-cli`.

This repository is generated from the Maimbrain monorepo; open issues here.

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option. Games you make with it are yours.
