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
| restyling the user's photo or sketch into the game (`--ref`) | text: never in generated images; draw it with the host fonts |

Run `mb models` first: models marked `✓` use the user's own key, `M` ones Maimbrain's keys (their account has access; the last line shows how much of this month's budget is left). Without either, `mb` says so. Tell the user what they'd need in one line: `mb keys set gemini` with billing on (docs/GENERATE.md "What to set up"), or ask the Maimbrain team for access to its keys and `mb login`. Then carry on with code-drawn assets rather than waiting. Don't use `--provider maimbrain` to get around the user's own key, and never pass `--allow-name` to slip a flagged name past the check (Maimbrain's keys refuse it anyway). You can still build the whole pipeline with `--provider mock` placeholders and swap them later with `mb regen <asset> --provider gemini`: `mb art check` lists every mock leftover. Never publish mock placeholders.

Check `mb models` (prices, which keys are set) before a batch, and tell the user roughly what it will cost; on Maimbrain's keys, also whether it fits what's left of their monthly budget. A typical 2D game is under a dollar: a hero, a few enemies or items, a background or three parallax layers, the icon (each ~$0.03–0.05 at the default model) and one or two music loops ($0.04 each). Use `--variants` and Pro models deliberately; every command refuses to spend more than $1 unless given `--max-cost`.

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

For sound, you can't listen: read the numbers each command prints (length, peak, loud RMS, the loop seam "clean" or "CLICK", the loop's bars and BPM) and run `scripts/check_audio.py` on the files (`--loops <name>` for loops whose names don't contain "music" or "loop"). Tell the user what you couldn't verify by ear, and that generated music should be heard once on a phone before publishing. For rhythm games, build the beat grid from the BPM `mb music` reports for the cut loop (it's derived from the loop's exact length), not the BPM you asked for.

## 5. Budgets

- Bundle ≤ 10 MB, startup set (wasm + `startup_assets`) ≤ 3 MB, ≤ 128 images, each ≤ 4096² (SPEC §1, §5.3). `mb art check` adds it up.
- Typical sizes after mb's PNG optimizer: a 128² sprite 5–20 KB (pixel art 1–3 KB), a 720×1280 background 300–900 KB, a 2-screen parallax layer 200–600 KB, a 30 s mono music loop at `-q 4` ~250 KB.
- Big backgrounds: `--colors 128` or `--max-kb 400` (quantizes until it fits; check the preview for banding on gradients), or generate at a smaller `--size` and draw it scaled up if the art is soft anyway.
- Many sprites: `mb art atlas games/<name> sprites` packs them into one PNG and writes `src/sprites.rs` with source rects; delete the individual PNGs (`--remove-inputs`) once the game draws from the atlas, or they ship too.
- Only list what the first seconds need in `startup_assets`; music and late assets load in the background.

## 6. Provenance

Each asset's sidecar (`assets/<file>.gen.json`) records prompt, model, seed, date, references and cost, and ships in the bundle so reviewers can see how the art was made. Don't delete or hand-edit sidecars; if you hand-edit a generated image, `mb art check` notes it. Commit `art/style.toml`, `art/ref/` and the sidecars; `art/raw/`, `art/variants/` and `art/preview/` are git-ignored.
