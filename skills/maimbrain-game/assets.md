# Art and sound: generated or code-drawn

A game's assets come from one of two places, and most games use both:

- **Generated** with `mb art`, `mb music` and `mb sfx`: an image or audio model makes it, `mb` post-processes it to the platform's rules and writes it into `assets/` with a provenance sidecar. Painterly backgrounds, characters with texture and personality, an icon that looks like a real game, a real music track. Costs a few cents a call and needs the user's API key, or an account the Maimbrain team has given access to Maimbrain's keys (then `mb login` is enough). Full reference: `docs/GENERATE.md` (in the kit: `docs/GENERATE.md`).
- **Code-drawn** with a script in `games/<name>/tools/` (`scripts/pixel.py`, `scripts/sfx.py`) or at runtime with mb2d shapes: free, exact, remixable, tweakable by numbers.

## Which to use

| Use generated for | Use code for |
|---|---|
| characters, creatures, props with detail or texture | anything that changes shape or color with state (draw it in greys and tint it; see SKILL.md §3) |
| backgrounds, parallax layers, tiles | geometry the rules depend on (hitboxes you must match exactly, platforms, grids, meters) |
| the icon | particles, glows, trails, screen flashes (mb2d/mb3d do these better live) |
| music: a real track with instruments | effects that must sit exactly on the beat or follow a pitch scale (sfx.py, tuned to the game's key) |
| sound effects with a real-world source (glass, wood, creatures, weather), when there's an ElevenLabs key | 8-bit blips, UI ticks, sweeps; any effect when there's no ElevenLabs key |
| restyling the user's photo or sketch into the game (`--ref`) | text: never in generated images; draw it with the game's fonts (`mb font add`, below) |

Run `mb models` first: models marked `✓` use the user's own key, `M` ones Maimbrain's keys (their account has access; the last line shows how much of this month's budget is left). Without either, `mb` says so. Tell the user what they'd need in one line: `mb keys set gemini` with billing on (docs/GENERATE.md "What to set up"), or ask the Maimbrain team for access to its keys and `mb login`. Then carry on with code-drawn assets rather than waiting. Don't use `--provider maimbrain` to get around the user's own key, and never pass `--allow-name` to slip a flagged name past the check (Maimbrain's keys refuse it anyway). You can still build the whole pipeline with `--provider mock` placeholders and swap them later with `mb regen <asset> --provider gemini`: `mb art check` lists every mock leftover. Never publish mock placeholders.

Check `mb models` (prices, which keys are set) before a batch, and tell the user roughly what it will cost; on Maimbrain's keys, also whether it fits what's left of their monthly budget. A typical 2D game is under a dollar: a hero, a few enemies or items, a background or three parallax layers, the icon (each ~$0.03–0.05 at the default model) and one or two music loops ($0.04 each). Use `--variants` and Pro models deliberately; every command refuses to spend more than $1 unless given `--max-cost`.

## Fonts

Type is part of the art. Pick the game's fonts in the art-direction step ([identity.md](identity.md)) and match the style guide to them: a slab western face wants a wood-and-brass palette, a ballpoint hand wants paper. `mb fonts` lists the 96 library fonts with mood, era, uses and pairings; `mb fonts --preview` draws a specimen sheet; `mb fonts --category pixel` or `--search horror` narrows it. `mb font add games/<name> <id>` downloads one from github.com/google/fonts (pinned, checksum-verified), bakes a subset atlas into `assets/fonts/<id>.mbf` with its license, and records it in `fonts.toml` so `mb build` re-bakes it if you change the entry. A font of your own (a .ttf/.otf you may redistribute) works the same: `mb font add games/<name> fonts-src/Mine.ttf --license fonts-src/LICENSE.txt`. Keep a game to 2–3 fonts of 25–60 KB each.

## 1. Write the style guide first

`mb art init games/<name>` writes `art/style.toml`. Fill it in from DESIGN.md before generating anything, because every image gets it:

- `style`: one or two concrete sentences: medium, shape language, line, light, mood. "Chunky flat vector shapes with thick dark-purple outlines, two-tone cel shading, warm dusk light" beats "cute and colorful".
- `keywords`: 3–6 short words that matter for *this feed*: readable silhouettes, high contrast, big shapes.
- `palette`: 5–8 hex colors, darkest first, with clear value steps (dark outline, two mid tones, a highlight, one accent that means "danger" or "reward"). Check that the player, the hazards and the background are different values, not only different hues.
- `avoid`: what would break the look (photorealism, gradients in the sky, busy detail).
- `pixel_art = true` (with `pixel_scale = 4`) for pixel games: every image is reduced to a grid of whole art pixels and stored at 4 texels per art pixel (SPEC §5.3: draw each art pixel 2 logical units wide). Usually with `palette_lock = true`.
- `[sprite] size`: the size your code draws sprites at × 2 (pixels per logical unit). The main character ≥ 1/3 of the screen width (120+ units → 240+ px); anything the player tracks ≥ ~40 units (80 px).
- `[music] prompt` / `[sfx] prompt`: the sound's palette (instruments, texture), so all tracks and effects belong together.

Generate the hero first, iterate until it's right, then add it to `references` (`references = ["assets/hero.png"]`): later generations match it.

## 2. Write prompts like briefs

Describe what it *looks like*, as something original: subject, pose or action, the one or two details that make it read. "A round moth with stained-glass wings spread wide, big curious eyes" — not "a cool moth character". The kind's framing (centered, margin, no ground, flat background, no text) is added for you; `--dry-run` shows the full prompt.

- Never name existing characters, franchises, games, brands or artists. `mb` refuses obvious names (`--allow-name` only for genuine false positives or the user's own IP) and the community guidelines reject copies. "In the style of <studio or living artist>" gets a warning: describe the qualities instead.
- One subject per sprite. For animation, use `mb art frames … --frames 4..8` ("the moth flapping its wings", "the frog mid-hop"): the frames come out the same size in equal cells, bottom-anchored. Look at them in sequence: if one frame is off, regenerate (consistency isn't guaranteed).
- Backgrounds: say what's *behind* the play ("a moonlit cathedral nave, stained-glass windows high up"); the prompt already asks for a calm, lower-contrast middle where the game happens.
- Music: genre, instruments, mood, energy, and `--bpm` if the game keeps time with it. "Bouncy marimba and hand claps, playful, light". The prompt already asks for instrumental, steady, no intro or ending. `--layers` gives a calm base and a bright high layer to fade in with intensity (the mothglass/summoned pattern).
- Effects: source, material, action, length. "A short bright glass chime with a soft attack", `--seconds 0.3`.

What the generators do that the docs don't say:

- **ElevenLabs** (`mb sfx`) allows 4 requests at once per key (shared with everyone using it: run `mb sfx` one at a time) and never returns less than ~0.5 s, so a click or tick comes with a tail: play it quieter or trim it. A harsh take: ask for "a soft attack, no sparkle, no shimmer, no high ring, one short note" and a mellow material (felt, wood, rubber). A late one: "starts immediately".
- **Lyria** (`mb music`) ignores the key you ask for (and drifts from the BPM): measure the key (`mix_check.py` prints it) and use the BPM `mb music` reports.
- **Characters**: generate every pose or expression of one character as one sheet with `mb art frames` (they stay the same character), then use the hero as `--ref` (or in `references`) for its color variants and everything after. When code depends on a shape, say it: "a perfect square exactly as wide as it is tall", "almost twice as wide as it is tall". For sprites, "floating on the flat background with no ground and no shadow beneath it" and "nothing peeks out behind it" (models add backing layers and drop shadows that key badly). Pick a `key` color the subject never contains (pure green for candy, magenta for foliage) and put that color in `avoid`.

## 3. Iterate cheaply

- `--dry-run` first when unsure: prompt and price, no call.
- `--variants 3`: three candidates and `art/variants/<name>-sheet.png` to compare them in one look; `mb art pick games/<name> <name> 2` keeps one.
- `mb regen games/<name>/assets/hero.png` — same recipe and seed; `--new-seed` for another take; `--seed N` to lock one you liked; `--prompt "…"` to change the description. The sidecar keeps everything else (size, grid, provider, model).
- `mb regen <asset> --reprocess` — redo only the post-processing from the saved raw output: free, offline. Use it after changing the palette, `colors`, `palette_lock` or `--tolerance`.
- Seeds are honored by Gemini/Lyria (best effort) and the mock, not by OpenAI or ElevenLabs: there the raw output in `art/raw/` is how you keep a result.
- After editing `art/style.toml`, `mb art check` lists assets made with the old version; regenerate the ones that matter.

## 4. Look at every result

You can't judge an image from its file size. After each generation, **open the preview** the command prints (`art/preview/<name>.png`: the asset over a checkerboard, a dark and a light ground, enlarged) and check:

- the silhouette reads at game size (squint: is it still a moth?), and against your background's value
- no leftover background: a halo or fringe of the key color around the edges (regenerate, or raise `--tolerance` and `--reprocess`), holes where the subject had the key's color (set a different `key` in the style guide)
- nothing cut off (the log warns when the subject touched the image edges), no stray text, letters or watermark-like marks
- it matches the hero and the palette
- frames: same size, same facing, a believable cycle
- tiles and layers: put two side by side in your head (or in the game) and look at the seam

Then look at it **in the game** (`mb serve` and screenshots, SKILL.md §4): scale, contrast against the background, the card overlays.

For sound, you can't listen: read the numbers each command prints (length, peak, loud RMS, the loop seam "clean" or "CLICK", the loop's bars and BPM) and run `scripts/check_audio.py` on the files (`--loops <name>` for loops whose names don't contain "music" or "loop"), then check how they sit together (§5). Tell the user what you couldn't verify by ear, and that generated music should be heard once on a phone before publishing. For rhythm games, build the beat grid from the BPM `mb music` reports for the cut loop (it's derived from the loop's exact length), not the BPM you asked for.

## 5. Mix the sound

You can't hear the mix, so measure it. `scripts/mix_check.py` renders a bot-played round with the real files the way the platform mixes (master 0.8 into a −9 dB limiter, 32 voices) and measures loudness K-weighted (like LUFS) per bus and per sound. The kits' sound modules record every event in test builds and `tests.rs` has a `mix_log` test that plays a bot round through them: `MB_MIX_LOG=/tmp/mix.log cargo test -p <crate> mix_log`, then `python3 <skill dir>/scripts/mix_check.py games/<name>/assets /tmp/mix.log` (`--fx-loop <name>` for a loop that is an effect, like a ticking clock). Keep that recorder when you change the sound; in a game without one, log `<seconds> <sound> <vol> <pitch> [play|loop|set|stop]` lines the same way.

Levels. Files are saved at −20 dBFS RMS for music and −14 for effects (`mb`, sfx.py), so effects play well under vol 1. The rules mix_check enforces:

| | rule (vs the music, in the same round) | typical vol |
|---|---|---|
| music | the reference: about −23 LUFS short-term; louder hits the limiter | 0.85–1.0 (a quiet stem more) |
| all effects together | music over them by ≥ +3 dB in the median 400 ms window, and ≥ −6 dB in 90 % of windows | |
| frequent (≥ 40 a minute: shots, coins, pops, thuds, ticks) | its own 3 s loudness ≥ 8 dB under the music, never peaking over it, under half the effects' energy, brightness ≤ 3.5 kHz, a soft attack | 0.15–0.3 |
| regular (6–40 a minute: a swap, a landing, a ding per answer) | ≥ 4 dB under, peaks ≤ +4 dB | 0.3–0.5 |
| rare and important (a hit, a fail, a level, a boss, a new best) | peaks ≤ +8 dB; may duck the music 3–6 dB (vol × 0.5–0.7) for under a second, never deeper | 0.5–0.8 |
| master limiter | pulls > 3 dB less than 5 % of the time (more pumps the music) | |

Variation and voices:

- Every frequent sound varies each play: level ±1–2 dB, and pitch ±3–5 % if it is unpitched. A tonal sound varies its level, its pitch ≤ ±1 % (±4 % is most of a semitone off key).
- Cap same-sound voices at 3–4 and rate-limit them (one per 60–100 ms; fold a burst into one sound). To steal a voice, `set` its vol to 0 and `stop` it a frame later (`stop` alone clicks). The platform's 32-voice limit steals the oldest one-shot, not the least important one.

Key and pitch:

- Pitch every tonal effect onto the music's notes: mix_check flags a tonal sound whose played notes clash with the measured key, and a music layer (a tension stem) that clashes with the main loop, with the shift that fits best.
- Pitch ladders (combo, streak) climb on the scale's notes, stay within ±7 semitones of the sample (further, it chipmunks or turns to sludge), and start over.
- Effects come from the music's world: the same instrument family, synthesized effects for synthesized music.

Before shipping: `check_audio.py` clean; `mix_check.py` clean on a typical bot round and on a long one, or each remaining flag explained in DESIGN.md; the title card sparse (cards play sound in the feed); and tell the user the mix was measured, not heard.

## 6. Budgets

- Bundle ≤ 10 MB, startup set (wasm + `startup_assets`) ≤ 3 MB, ≤ 128 images, each ≤ 4096² (SPEC §1, §5.3). `mb art check` adds it up.
- Typical sizes after mb's PNG optimizer: a 128² sprite 5–20 KB (pixel art 1–3 KB), a 720×1280 background 300–900 KB, a 2-screen parallax layer 200–600 KB, a 30 s mono music loop at `-q 4` ~250 KB.
- Big backgrounds: `--colors 128` or `--max-kb 400` (quantizes until it fits; check the preview for banding on gradients), or generate at a smaller `--size` and draw it scaled up if the art is soft anyway.
- Many sprites: `mb art atlas games/<name> sprites` packs them into one PNG and writes `src/sprites.rs` with source rects; delete the individual PNGs (`--remove-inputs`) once the game draws from the atlas, or they ship too.
- Only list what the first seconds need in `startup_assets`; music and late assets load in the background.

## 7. Provenance

Each asset's sidecar (`assets/<file>.gen.json`) records prompt, model, seed, date, references and cost, and ships in the bundle so reviewers can see how the art was made. Don't delete or hand-edit sidecars; if you hand-edit a generated image, `mb art check` notes it. Commit `art/style.toml`, `art/ref/` and the sidecars; `art/raw/`, `art/variants/` and `art/preview/` are git-ignored.
