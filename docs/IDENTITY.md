# Visual identity: why Maimbrain games look alike

Measured 2026-10-07 to back up the complaint that "all the games Claude is producing look identical between different creators".

## What was measured

- **Live feed:** all 27 games on `maimbrain.com/api/v1/feed`, from 4 creators: fritz (10), terracatta (9), ghostpeep (4) and thewildsamus (4). Each bundle was run in the desktop preview (headless Chrome, 360×640 logical). For each one I took the title card after 90 held frames, a mid-play frame, and the game-over card when scripted input could reach it (17 of 27). The Rust source shipped inside each bundle was searched for the calls that draw this chrome.
- **Repo:** `games/*` and the two skill templates (`mb new`, `mb new --3d`). Six repo games are the same builds as their live versions. `hang` (watchdog test) and `mb3d-demo` (tech demo) are not counted.
- Contact sheets: [before-live.jpg](identity/before-live.jpg) (27 live games) and [before-repo.jpg](identity/before-repo.jpg) (repo-only games, the UI-kit Stack, the templates and the gallery).

The games differ in subject, art and even renderer: there are pixel-art sprites (17), 3D scenes (7) and flat vector (3). The sameness is in the chrome: type, title layout, HUD, game-over card and effects. A lot of that chrome is the same code, copied from game to game.

## What repeats (live feed, n = 27 unless noted)

| Element | Count |
|---|---|
| Title set in the 5×7 pixel font (`Font::Pixel`) with a hard drop shadow or 1-px outline | 19/27 (7 use Inter Bold; Live Wire draws its own font) |
| Pixel font somewhere on screen (title, tagline, HUD, hints) | 24/27 |
| A `text_c` helper that draws text, then a shadow copy offset by `size/8` down-right in near-black plum (`0x1a0d16e0`-ish) | 17/27, in all 4 creators |
| An `fx.rs` module with the same contents (`Fx` struct holding particles, popups and shake, `ease_out_back`, `text_c`, `text_pop`, `alpha`, `mix`) | 18/27, in all 4 creators |
| Title centred in the top ~20% of the screen | 23/27 |
| One-line all-caps tagline under the title | 24/27 |
| ...and that tagline is "DON'T …" or "HOW … ?" | 11/27 |
| Score as a big white number at top centre, with a small caption under it (M, FEET, LEVEL 1, BEST n) | 26/27 |
| All-caps "VERB TO VERB" hint mid-screen during play (TILT TO STEER, TAP TO DROP, DRAG TO DRIVE…) | 21/27 |
| ...with a drawn phone-tilt glyph | 6/27 |
| Game-over card stacked as one-word headline + "!", then a huge number, then a BEST line, all in the upper half | 15/17 reached |
| ...on a dark translucent box or band | 10/17 |
| "NEW BEST" string in the source | 25/27 |
| Purple or violet as the dominant background (dusk skies, plum walls, space) | 12/27 |
| Vertical gradient backdrops (`rect_gradient`) | 23/27 |
| Screen shake | 25/27 |
| Back-out overshoot easing for pops and entrances | 18/27 |

Headlines from the game-over cards: SO CLOSE!, ROOFED!, TERMINATED, SPLAT!, LIGHTS OUT, FIRED!, CAT-ASTROPHE!, STARVED!, WAX FLOOD!, SHATTERED, TABLE FLIPPED!, SHOT DOWN, PARTY WIPED, TIPPED!, CLOSED EARLY!, FACEPLANT!. Repo games add TOPPLED!, OUT OF BALLS, TRIPPED!, POP!, ESCAPED!.

## Where it comes from

- **The host has only three fonts:** Inter, Inter Bold and the pixel font (`gfx2d::Font`). Inter has no personality at title sizes and the pixel font has a lot, so most games reach for the pixel font, and every game that does looks related. Only Live Wire built its own letterforms (skewed glyph meshes in `font.rs`).
- **The skill's examples get copied word for word.** The skill's sample lines ("HOW HIGH?", "DON'T LET BRIAN GO THIRSTY.", "SPLAT!", "NEW BEST!", "BUBBLE" / "don't let it pop") come back as taglines and headlines. Its card rules (hook plus tagline on the title, score plus best on game-over) are good rules, but they leave every card with the same layout.
- **The UI kit is next.** No live game uses the kit yet (`TitleCard`, `ResultsCard` and `Hud`: 0/27). The repo Stack, the gallery and both templates do use it, and the skill tells authors to "copy their shape" and not hand-roll text. Its default is `Theme::candy()`: Inter Bold titles with a dark outline and soft shadow, letters dropping in with `BackOut` and then bobbing, a lowercase tagline, a plum gradient (`#3a1c71`→`#1a0b3a`), a rounded 22-px plum panel, a pink round retry button, a gold trophy "BEST" pill, and a confetti "NEW BEST!" ribbon. The 3D template uses `night()`, which is the same layout in navy and teal. Unless authors change it, every new game will get this look.

## The house look, in one paragraph

A Maimbrain game opens on a dusky purple or night scene. Its name sits centred near the top, in chunky pixel capitals with a hard dark drop shadow, or in outlined Inter Bold that drops in letter by letter and bobs. Under the name is a short all-caps line, often "DON'T …" or "HOW HIGH …?". In play, a big white score sits at top centre with a tiny caption under it, and an all-caps "TILT TO …" or "TAP TO …" hint floats mid-screen. Things pop in with an overshoot, the screen shakes on every hit, and particles burst out. When you lose, a dark translucent box appears in the upper half with a one-word exclamation in red or orange, a huge number, and "NEW BEST!" in yellow, sometimes with confetti. If your game matches most of that, players will read it as one more of the same.

## What stands out

- **Live Wire** (fritz): synthwave neon, its own skewed 3D glyph font, a chrome-gradient title.
- **Hold Your Horses!** (thewildsamus): the score sits in a prize rosette, and the cause of failure is written on a pink pill ("SPOOKED BY A DUCK!").
- **Pea Hustle** (ghostpeep): a 3D character who talks in speech bubbles, and poker chips used as the controls.
- **Holding Space** (thewildsamus): a board-game layout with room cards and an item tray. The HUD is part of the scene.
- **Robot Escape** (fritz): the game-over card is a red ribbon banner, and the truck that caught you fills the frame.
- **Cairn** (fritz): a misty grey, painterly scene with a left-aligned title.
- **Orbit** and **Stack** (terracatta): plain and minimal. Calm, but plain is not the same as distinctive.

## Checklist: does this look like every other Maimbrain game?

Answer yes or no. Each "yes" is a place to try something else.

1. Is the title in the pixel font with a hard drop shadow, or in outlined Inter Bold that drops in letter by letter?
2. Is the title centred in the top fifth, with a one-line all-caps tagline under it?
3. Does the tagline start with "DON'T" or "HOW", or reuse a line from the skill's examples?
4. Is the background a vertical gradient, or purple, plum, dusk or deep space?
5. Is the score a big white number at top centre with a small caption under it?
6. Is the game-over card headline, then big number, then "NEW BEST!", on a dark translucent box in the upper half?
7. Is the headline one word with an exclamation mark (SPLAT!, TIPPED!, FIRED!)?
8. Did you leave the kit on `Theme::candy()` or `night()`, or copy `fx.rs` and its `text_c` and `text_pop` from another game?
9. Do pops overshoot with back-out easing, does every hit shake the screen, and do particles burst on every score?
10. Could the title, HUD and game-over card move onto another game's art with nothing looking out of place?

If question 10 is "yes", the chrome isn't telling players anything about this game. Make the type, the frames and the game-over card come from the game's own world: a sign, a rosette, a receipt, a scoreboard.

## What the kit does about it

- **Fonts.** Games ship their own fonts (`stdlib = { mb2d = 2 }`, SPEC §5.3). `mb fonts` lists a library of 96 OFL/Apache families across 21 categories, from geometric sans to blackletter, ten pixel faces, stencils and westerns. `mb fonts --preview` draws a specimen sheet of them. `mb font add` bakes one into a small subset atlas, and `Font::asset` draws with it anywhere.
- **Themes with a look, not just colors.** A theme now carries shape language, borders, shadows, textures, backdrops, title, results and HUD layouts, and a motion personality. There are 17 identities that look nothing alike ([identity/identities.jpg](identity/identities.jpg)), and `Theme::from_identity("…")` composes one from a description (docs/UI.md).
- **New games don't start alike.** `mb new` starts each game from a random identity and bakes its fonts. The skill's art-direction step (identity.md) has the agent write down a reference, palette, font pairing, shape language, motion and one signature idea, then check the result against the checklist above. `mb build` warns when a game uses only the built-in fonts or a stock theme as-is.
- **Before and after.** [identity/before-after.jpg](identity/before-after.jpg) shows Stack (now a Bauhaus poster), the 2D template (`mb new --identity comic`) and Orbit (an 18th-century star chart), at the same frames as above.

