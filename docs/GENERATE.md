# Generated art and sound (`mb art`, `mb music`, `mb sfx`)

`mb` can ask an image or audio model for a game's assets and turn what comes back into files that drop straight into `assets/`: exact sizes, transparent sprites, pixel grids, the game's palette, seamless tiles and layers, PNGs within budget; music cut to a seamless whole-bar loop, effects trimmed, everything mono Ogg Vorbis at the platform's loudness (SPEC §5.6). Every asset gets a provenance sidecar, and a per-game style guide keeps one game's art consistent.

You can still draw art and synthesize sound with scripts (the maimbrain-game skill's `pixel.py` and `sfx.py`). The skill explains when to use which.

## Quick start

```sh
mb keys set gemini                     # paste a Gemini API key (or export GEMINI_API_KEY)
mb art init games/moth                 # games/moth/art/style.toml: style words, palette, references
mb art sprite games/moth hero "a moth with stained-glass wings, wings spread"
mb music games/moth music "dreamy harp and glass chimes over a soft pulse" --bars 8
mb sfx games/moth chime "a bright glass chime, short"      # needs an ElevenLabs key
mb art check games/moth                # budgets, and what made each asset
```

If the Maimbrain team has given your account access to **Maimbrain's keys**, you don't need a key of your own: sign in with `mb login` and the same commands work (see [Maimbrain's keys](#maimbrains-keys)).

No key yet? Add `--provider mock` to any command: the offline mock provider makes deterministic placeholder art and sound, so the whole pipeline can be tried (and is tested) without a network. `mb art check` flags mock assets so they don't ship by accident. `--dry-run` prints the exact prompt and the estimated cost without calling anything, key or not.

## Commands

All image commands take the game directory first, then the asset name (output `assets/<name>.png`), then a plain description of the subject.

| Command | Makes | Defaults |
|---|---|---|
| `mb art init <game>` | `art/style.toml`, `art/ref/`, `art/.gitignore` | |
| `mb art sprite <game> <name> "<prompt>"` | a sprite on a transparent background | 128×128 |
| `mb art frames <game> <name> "<prompt>" --frames N [--cols C]` | an animation sheet of equal cells (one row up to 8 frames) | 128×128 cells, bottom-anchored |
| `mb art background <game> <name> "<prompt>"` | an opaque full-screen background | logical size × 2 (720×1280) |
| `mb art layers <game> <name> "<prompt>" --count N [--screens 2]` | parallax layers `<name>_0…` far to near; layer 0 opaque, the rest transparent; all wrap horizontally; later layers use layer 0 as a style reference | 2 screens wide |
| `mb art tile <game> <name> "<prompt>"` | a seamless square texture | 64×64 |
| `mb art ui <game> <name> "<prompt>"` | a button, panel or badge on a transparent background | 256×128 |
| `mb art icon <game> "<prompt>"` | `icon.png`, 256×256, opaque | |
| `mb art pick <game> <name> <n>` | promotes candidate n from `--variants` to `assets/` | |
| `mb art atlas <game> <name> [pngs…] [--trim] [--padding 2]` | packs PNGs into `assets/<name>.png` and writes `src/<name>.rs` (`pub const HERO: [f32; 4] = [x, y, w, h];`, the same shape as `pixel.py`) | every generated sprite and UI PNG |
| `mb art check <game>` | sizes against the bundle (10 MB) and startup (3 MB) budgets, image limits, mono audio, who made each asset, stale style, mock leftovers, money spent | |
| `mb music <game> <name> "<prompt>"` | a looping track `assets/<name>.ogg` | 8 bars, 44.1 kHz, `-q 4` |
| `mb sfx <game> <name> "<prompt>"` | a sound effect | trimmed, 44.1 kHz |
| `mb regen <asset>` | the same asset again from its sidecar | same seed |
| `mb keys set\|list\|remove <provider>` | API keys | |
| `mb models` | every model with its price, whether your key is set, and what Maimbrain's keys cover for you | |

Shared image options: `--size WxH` (output pixels; the canvas is 2 px per logical unit, so a 60-unit-wide sprite is 120 px), `--grid WxH` (pixel art: the art-pixel grid; output = grid × the style's `pixel_scale`), `--ref <photo>` (repeatable: redraw this photo's or sketch's subject in the game's style), `--seed N`, `--variants N` (N candidates in `art/variants/` plus a contact sheet), `--provider` (`maimbrain` forces Maimbrain's keys), `--own-keys` (never Maimbrain's keys), `--model`, `--res 0.5K|1K|2K|4K`, `--quality low|medium|high` (OpenAI), `--anchor center|bottom`, `--margin PX`, `--colors N`, `--max-kb N` (quantize until the PNG fits), `--keep-background`, `--tolerance` (keying), `--out PATH`, `--no-style`, `--allow-name WORD`, `--max-cost USD`, `--dry-run`.

Music options: `--bars N`, `--bpm B` (asked for, and used instead of detection), `--seconds S` (free-length loop for pieces without a steady beat), `--layers` (also writes `<name>_base.ogg` and `<name>_hi.ogg`), `--split-hz 700`, `--once` (a jingle or sting, not a loop), `--negative` (Vertex lyria-002), `--rate`, `--quality`. Effects: `--seconds`, `--loop`.

`mb regen <asset>` options: `--seed N` or `--new-seed`, `--prompt "…"`, `--provider`/`--model`, `--variants N`, `--reprocess` (redo only the post-processing from the saved raw output: free and offline; use it after changing the palette, colors, size rules or keying), `--dry-run`, `--max-cost`.

## Providers and prices (October 2026)

| Provider | Images | Music | Effects | Seeds | Key |
|---|---|---|---|---|---|
| `gemini` (default) | `gemini-nano-banana-2.1` $0.034 (1K) / $0.050 (2K); `gemini-3.1-flash-lite-image` $0.034 (1K only); `gemini-3.1-flash-image` $0.045 (512 px) – $0.151; `gemini-3-pro-image` $0.134 (1K/2K) | `lyria-3-clip-preview` $0.04 per 30 s clip (default); `lyria-3.5` $0.08 per song (WAV) | – | yes (best effort) | `GEMINI_API_KEY` |
| `vertex` | the same Gemini image models | Lyria 3 Clip $0.04; `lyria-002` $0.06 (30 s WAV, honors seed and a negative prompt) | – | yes | `GOOGLE_CLOUD_PROJECT` + `gcloud auth` |
| `openai` | `gpt-image-2.5-flare` (default), `gpt-image-2.5-sunburst`: $30 per 1M image output tokens, ≈ $0.01–0.13 per 1K image by quality; real transparent backgrounds | – | – | no | `OPENAI_API_KEY` |
| `elevenlabs` | – | `music_v2_5` $0.15/min (exact length, guaranteed instrumental) | `eleven_text_to_sound_v2` $0.12/min, 0.5–30 s, can loop | no | `ELEVENLABS_API_KEY` |
| `mock` | placeholders | a test song | a test effect | yes | none |

Sources: [Gemini API pricing](https://ai.google.dev/gemini-api/docs/pricing), [Vertex AI pricing](https://cloud.google.com/vertex-ai/generative-ai/pricing), [OpenAI pricing](https://developers.openai.com/api/docs/pricing), [ElevenLabs API pricing](https://elevenlabs.io/pricing/api). `mb models` prints the table mb uses. Every command prints its estimated cost before calling (OpenAI responses report their token usage, and mb prints the actual cost), refuses when one command would cost more than **$1.00** (`--max-cost`, or `MB_MAX_COST`), and records the estimate in the sidecar; `mb art check` adds up what a game's assets cost.

Why these defaults: Nano Banana 2.1 is Google's current image workhorse and costs the same at 1K as the Lite model, while following style references and multi-image instructions much better; Pro is 4× the price for briefs that need it. Lyria 3 Clip always makes 30 s, which holds an 8-bar loop at about 80 BPM or faster (4 bars below that), for $0.04. Google's image and music APIs embed a SynthID watermark; the sidecar notes it. Gemini 2.5 Flash Image (the original "Nano Banana") was shut down on 2 October 2026 and isn't offered.

Google has no sound-effect model in its APIs, so `mb sfx` uses ElevenLabs, whose free plan includes a few effect generations a month (check its license terms: the free plan isn't licensed for commercial use). Without an ElevenLabs key, make effects procedurally with the skill's `scripts/sfx.py` (8-bit voices, sweeps, noise, a step sequencer): it already meets the same loudness and encoding rules, and costs nothing.

### What to set up

- **Gemini API (recommended; images and music):** sign in at [Google AI Studio](https://aistudio.google.com/apikey), create an API key, and **enable billing on its Google Cloud project** (AI Studio → Billing): the image and music models have no free tier on the API (you can try them for free in the AI Studio web UI, not through the API). New accounts may be asked to prepay $10. Then `mb keys set gemini` or `export GEMINI_API_KEY=…`.
- **Vertex AI (alternative; where Google Cloud free-trial credits apply):** a Google Cloud project with the Vertex AI API enabled and the [gcloud CLI](https://cloud.google.com/sdk/docs/install) signed in (`gcloud auth login`). `mb keys set vertex` takes the **project id**; mb gets a short-lived token from `gcloud auth print-access-token` for each call (or set `VERTEX_ACCESS_TOKEN`). `GOOGLE_CLOUD_LOCATION` defaults to `global` (lyria-002 uses `us-central1`). New Google Cloud customers get $300 in credits, which as of 2026 apply to Vertex AI but not to Gemini API keys made in AI Studio.
- **ElevenLabs (sound effects; free plan available):** create an account, then an API key at [elevenlabs.io/app/settings/api-keys](https://elevenlabs.io/app/settings/api-keys) with access to Sound Effects (and Music, for music). `mb keys set elevenlabs`.
- **OpenAI (optional, for native transparency):** an API key at [platform.openai.com/api-keys](https://platform.openai.com/api-keys); GPT Image models may require [organization verification](https://help.openai.com/en/articles/10910291-api-organization-verification). `mb keys set openai`.

Keys are read from the environment first, then from `~/.config/maimbrain/keys.json` (`%APPDATA%\maimbrain\keys.json` on Windows; mode 0600), next to `mb login`'s credentials. Nothing is written into a game or repository. `mb keys list` shows which are set (masked) and where each comes from; `mb doctor` reports them too.

### Maimbrain's keys

The Maimbrain team can let individual creators generate on **Maimbrain's** Gemini and ElevenLabs keys instead of their own. Access is granted per account by an admin (there's no way to request it from `mb`: ask the team), and comes with a monthly budget, $10 unless the team sets another, that resets on the 1st (UTC).

Which keys a command uses:

1. **Your own key** for the chosen provider, if you have one (environment or `mb keys`). You pay; nothing changes.
2. Otherwise, if you're signed in (`mb login`) and the server says your account has access, **Maimbrain's keys**: mb sends the provider request to the server, which calls the provider and returns the raw image or sound. Everything else (keying, fitting, loops, loudness, the sidecar) still happens on your machine. The command prints `keys: Maimbrain's (@you) · $1.20 of $10.00 used this month` before the call and the new total after it.
3. Otherwise the usual "set a key" message, which also says how to get access.

Without a provider named, mb picks the first provider you have a key for (images: gemini, vertex, openai; music: gemini, vertex, elevenlabs; effects: elevenlabs), then the first of those Maimbrain's keys cover.

| To | Do |
|---|---|
| never use Maimbrain's keys | `--own-keys`, or `MB_KEYS_SOURCE=own` |
| always use Maimbrain's keys, even with a key of your own | `--provider maimbrain` (the default model for the kind), or `MB_KEYS_SOURCE=maimbrain` with `--provider gemini`/`elevenlabs` to pick one |
| see your access and this month's spend | `mb models` (models marked `M` are covered) or `mb doctor` |

What they cover: `gemini` images (Nano Banana 2.1, 2 Lite, 2 and Pro, at 0.5K–2K; not 4K) and music (Lyria 3 Clip, Lyria 3.5), and `elevenlabs` sound effects and music. Not `vertex` or `openai`: use your own key for those. Each call is charged against your budget at the list price above (the server's estimate, recorded in the sidecar as `cost_usd`, with `"via": "maimbrain"`); failed calls cost nothing. mb refuses up front when its estimate is more than what's left; the server refuses (HTTP 402) when the budget is spent, and allows two calls at a time and a few dozen every ten minutes.

The server runs the same originality check as mb on every prompt, and `--allow-name` isn't honored on Maimbrain's keys: if you own the rights to a flagged name, use your own key. The server keeps a record of each call (who, which model, the estimated cost, and a hash of the prompt, not the prompt itself).

### Adding a provider

`crates/mb-cli/src/generate/provider.rs` defines the `Provider` trait (`image`, `audio`), the request types, the model catalog with prices and a `Transport` trait for HTTP. A provider is one file that builds a request body, posts it through the transport and decodes the response (`google.rs`, `openai.rs`, `elevenlabs.rs`). Its tests replace the transport with canned responses, so they check request bodies and response parsing with no network.

`maimbrain.rs` is the provider that sends the same request to the Maimbrain server instead (and decides which keys a call uses). To offer a provider or model on Maimbrain's keys too, add it to `SERVER_PROVIDERS` there and to the server's `Generation::Catalog` and `Generation::Client` (`server/app/services/generation/`, docs/SERVER.md "Generation"); keep the two price tables in step.

## The style guide: `art/style.toml`

```toml
name = "Mothglass"
style = "luminous stained glass at night: jewel-toned panes, dark lead outlines, soft inner glow"
keywords = ["readable silhouettes", "flat color", "high contrast"]
avoid = ["photorealism", "busy detail"]       # text, watermarks, logos and borders are always avoided
palette = ["#140c26", "#3a2466", "#7b4fd6", "#f2c45a", "#ffe9b0", "#e2557a"]   # darkest first
palette_lock = false      # snap every pixel to the palette (good for pixel art)
pixel_art = false         # reduce to a grid of art pixels…
pixel_scale = 4           # …stored 4 texels wide (SPEC §5.3), drawn 2 units per art pixel
colors = 0                # quantize every asset to at most N colors (0: off)
references = ["art/ref/mood.png", "assets/hero.png"]   # up to 3 style references sent every time
key = "auto"              # flat background color asked for on transparent assets
# provider = "gemini"
# model = "gemini-nano-banana-2.1"
# resolution = "1K"

[sprite]                  # per kind: extra prompt words and a default size
prompt = "side view, facing right"
size = [160, 160]

[icon]
prompt = "the hero's face, three-quarter view"

[music]                   # added to every mb music prompt
prompt = "glassy bells, warm pads, gentle pulse"

[sfx]                     # added to every mb sfx prompt
prompt = "glassy, crisp, short tails"
```

Sections: `[sprite]`, `[frames]`, `[background]`, `[layer]`, `[tile]`, `[ui]`, `[icon]`, `[music]`, `[sfx]`. Every prompt is built the same way: what the asset is (per kind), the kind's words, `style`, `keywords`, the pixel-art grid, the palette, how to use the reference images, the background rule for transparent assets, what to avoid, and "an original design". `--dry-run` shows it. Each sidecar records the style file's hash, and `mb art check` flags assets made with an older version of it (`mb regen` refreshes them).

Once one asset looks right (usually the hero), add it to `references`: everything after it matches it.

## Post-processing

Images (`crates/mb-cli/src/generate/art.rs`, `img.rs`):

1. **Request**: the nearest supported aspect ratio to the target (`1:1`, `2:3`, `3:2`, `3:4`, `4:3`, `4:5`, `5:4`, `9:16`, `16:9`, `21:9`; frames are asked for in a near-square grid), at `1K` (backgrounds and layers `2K`), with the style references first and `--ref` photos last.
2. **Transparency** (sprites, frames, UI, near layers): the prompt asks for one flat `key` color (by default whichever of magenta, green, cyan or blue is farthest from the palette). mb estimates the real background from the image border, clears pixels close to it, computes each edge pixel's alpha by projecting its color between the background and the nearest solid subject color (which also removes the background's tint, "despill"), and drops specks. A provider's real alpha (OpenAI) is kept as is. Warnings say when the background wasn't flat or the subject touches the edges.
3. **Fit**: sprites and UI are trimmed and scaled into the exact size with a margin, centered or bottom-anchored; backgrounds and icons are cropped to the aspect ratio and resized (Lanczos, in premultiplied alpha).
4. **Frames**: the sheet is split into its connected shapes (small bits join the nearest frame) in reading order, falling back to an even grid; every frame gets the same scale and is anchored the same way in equal cells.
5. **Pixel art** (`--grid` or `pixel_art`): the content is fitted at 8 texels per art pixel, each cell becomes opaque when at least half covered and takes the median color of its middle (so a model's blended "pixels" don't muddy it), then it's scaled up by whole texels.
6. **Seamless** (tiles both ways, layers horizontally): the image is made a band wider, and the band is joined onto the opposite edge at the column (row) where the two match best, with a short blend.
7. **Color**: `palette_lock` snaps to the palette (Oklab distance), `colors`/`--colors` quantize (median cut, alpha included).
8. **PNG**: the smallest of indexed (at 1, 2, 4 or 8 bits, with tRNS), RGB and RGBA encodings with several filters at maximum compression. With `--max-kb`, it quantizes to 256, 128, … colors until the file fits; otherwise it warns about big files.

Audio (`sound.rs`, `audio.rs`):

1. Decode (WAV, MP3, Ogg or raw PCM), mix to mono, resample (windowed sinc) to `--rate`.
2. **Loops**: detect the tempo from the onset envelope's autocorrelation (or use `--bpm`); fit the largest whole number of 4/4 bars up to `--bars` after skipping the intro and the last second; search start points where what follows the loop's end best matches what follows its start (loudness contour over ~1 s, then waveform at sample level within ±0.4 % of the length); cross-fade the seam over 30 ms. The log and sidecar give the loop's length, bars, the BPM implied by the final length (use it for a beat grid), where it starts in the source and the seam check. Without a steady beat, `--seconds` cuts a free-length loop the same way.
3. **Layers** (`--layers`): a Linkwitz-Riley low-pass at `--split-hz`, run over the loop twice so its seam is seamless, gives `_base`; `_hi` is the rest, so base + high = the full mix. Both share one gain.
4. **Effects**: trim leading and trailing silence (3 ms pre-roll, 8 ms fade), or cross-fade the end into the start for `--loop`.
5. **Levels**: the same rules as `sfx.py`: RMS of the loud two thirds to −20 dBFS (music) or −14 dBFS (effects), a look-ahead limiter holding peaks under −3 dBFS.
6. **Encode** with `oggenc -q <quality>`, decode the result, and re-encode quieter if Vorbis pushed a peak past −1 dBFS; report length, peak, loudness and the loop seam (as `check_audio.py` does). Without `oggenc` the processed WAV is kept in `art/raw/`.

## Originality and provenance

Prompts (and the style guide) are checked against a list of well-known characters, franchises and brands; a match is refused with a message explaining why (the community guidelines don't allow someone else's IP, and reviewers reject it). Named living artists and studios in "in the style of …" get a warning. A false positive, or IP you own, goes through with `--allow-name <word>`, and the sidecar records that. The list catches the obvious cases only; it doesn't clear anything.

Every generated asset has a sidecar, `<asset>.gen.json` (`assets/hero.png.gen.json`; the icon's is `icon.png.gen.json`): the generator and mb version, date, provider, model, the user's prompt and the full prompt sent, the seed and whether the provider honors seeds, reference files and the style guide with SHA-256 hashes, the estimated and actual cost, the raw output's path and hash, the output's hash, names let through with `--allow-name`, any text the model returned, the resolved recipe `mb regen` replays, facts about the result (size, keyed color, loop points, BPM, loudness) and a provider note (SynthID). Sidecars in `assets/` are packed into the bundle (they're small JSON files), so a moderator can see how a game's assets were made.

`art/raw/` keeps every provider output (`<name>-<seed>.png|mp3|wav`), `art/variants/` the candidates, and `art/preview/` each image over a checkerboard, a dark and a light ground, enlarged: the file to look at. `mb art init` git-ignores all three; they're large and reproducible.

## Testing

`cargo test -p mb-cli` runs everything offline: image processing (keying and despill, pixel grids, palettes, quantizing, seamless wrapping, PNG encodings), audio (decoding, resampling, levels, tempo detection, loop cutting, layers, Ogg encoding when `oggenc` is installed), the IP check, keys, cost estimates, each provider's request bodies and response parsing against canned responses, and end-to-end runs of every command with the mock provider (`crates/mb-cli/tests/generate.rs`). Which keys a command uses is tested against a fake Maimbrain server on localhost (`crates/mb-cli/tests/maimbrain_keys.rs`, and the unit tests in `generate/maimbrain.rs`); `MB_CONFIG_DIR` keeps the tests away from your real `mb login`.
