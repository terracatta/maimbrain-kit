# Maimbrain creator kit

Make games for [Maimbrain](https://maimbrain.com): tiny games people swipe through on their phones. Games are Rust → WebAssembly; you describe one, and your agent builds it with the `maimbrain-game` skill.

## Get started

```sh
curl -fsSL https://maimbrain.com/install.sh | sh   # the mb tool
mb doctor                                          # checks Rust + the wasm32 target
mb login                                           # sign in to maimbrain.com
```

In Claude Code:

```
/plugin marketplace add terracatta/maimbrain-kit
/plugin install maimbrain@maimbrain-kit
```

Then ask: *"Use the maimbrain-game skill to make a game: …"*. Your agent will run:

```sh
mb new mygame        # a game crate from the template, with your id
mb serve mygame      # preview at http://127.0.0.1:8765 (reload to rebuild)
mb publish mygame    # upload for review; track it at maimbrain.com/create
```

## What's here

| Path | What |
|---|---|
| `skills/maimbrain-game/` | The skill: workflow, engagement guide, template, `pixel.py`, `sfx.py`, `check_audio.py` |
| `docs/SPEC.md` | The game spec (manifest, host API, limits) |
| `sdk/maimbrain/` | The Rust SDK (games depend on it via git) |
| `crates/mb-cli/` | The `mb` tool |
| `crates/mb-format/` | The bundle format and validator (the same one the server runs) |

Build `mb` from source: `cargo install --git https://github.com/terracatta/maimbrain-kit mb-cli`.

This repository is generated from the Maimbrain monorepo; open issues here.

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option. Games you make with it are yours.
