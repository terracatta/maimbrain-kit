# What makes a Maimbrain game engaging

The audience is someone swiping a vertical feed on their phone, one-handed, often with sound off, deciding in about a second whether your game is worth a tap. They didn't choose your game and won't read anything. They will instantly feel whether it responds to them. Everything below follows from that.

## 1. The first second: the card sells the game

The title screen isn't a menu. It's the feed card: an advertisement playing live, and an ad, not a tutorial. A card takes no input (no touches, no sensors), so it must never ask for an in-game action. "BLOW!", "TILT!", "SHAKE!", "DRAG" or an animated phone icon on a card invite something that does nothing, and the player feels the game is broken. Its job is to make a stranger *want* to play. Tapping to play is the platform's job (it teaches that itself), so the card doesn't say "TAP" either.

- **Show the game, not a logo.** The first frame shows the character and the play space mid-action: ball mid-bounce, beer mid-pour, tower mid-wobble. The title text is small; the scene is big.
- **Motion from frame one.** Something on the card is always moving (idle animation, swaying object, blinking eyes). A static card reads as a picture and gets swiped past.
- **A clear hook.** One funny or intriguing thing a stranger gets in one glance: a character with a face and a problem.
- **Stakes and drama, not steps.** Loop the most tempting moment: a near-miss, the wobble before the tower falls, the candle that almost relights, the face just before disaster. Tease the payoff without resolving it.
- **One line of desire, if any**: a tagline or challenge rather than an instruction. "90 CANDLES. ONE GRANDMA." / "DON'T LET BRIAN GO THIRSTY." / "HOW HIGH?" Add the player's best ("BEST 42") once they have one: it's a reason to come back.
- **How to play waits for play.** Every "how to" (BLOW, TILT TO POUR, the phone icon, arrows, ghost fingers) appears in the first seconds after the tap, when the action actually works (§2).

## 2. Instant, obvious interactivity

- **The first tap starts playing**: no menu, no "press start", no settings, no tutorial screen. The tap that entered play can be the first move.
- **Respond within one frame**, every time, on every input: something moves, squashes, flashes or makes a sound. A tap with no visible response feels broken.
- **Teach by doing, with on-screen affordances**, not words:
  - Arrows, ghost fingers, dotted paths and target rings show *where* and *which way*.
  - A pulsing outline marks what to touch; an arrow shows the direction to swipe or tilt.
  - A live indicator mirrors the control (a needle, a tilting glass, a power bar), so the player sees their input's effect even before it matters.
  - Hints persist until the player performs the action once, then fade. Bring them back if the player stalls (no input for ~3 s).
- **≤ 5 words of text at a time**, big (pixel font 16+), with a dark shadow or panel behind it. Verbs, not explanations: "TILT TO POUR", "TAP TO JUMP".
- **The first 3 seconds always succeed.** The opening is a no-fail warm-up: generous timing, slow pace, a guaranteed early win (first coin, first sip, first block placed) with a reward burst. The player should feel competent before they feel challenged.

## 3. Juice: make every action feel good

Juice is the feedback layered on an action so it feels physical. It's the difference between "works" and "fun".
- **Squash and stretch** on impacts and launches; **anticipation** (wind-up) before big moves.
- **Particles** for every success (sparkles, splashes, crumbs) and a bigger burst for milestones.
- **Screen shake**, small (2–4 units, 0.1–0.2 s) for hits, bigger for failure. Never constant.
- **Pops and easing**: numbers scale up then settle when they change; things ease in and out, never snap (except to show impact).
- **Sound on every action**, pitched up with streaks/combos (each consecutive success a semitone higher feels great).
- **Haptics** in step with the big visual beats: a tick on each small success, a thud on impact and failure. Silent mode must still feel physical.
- **Hit-pause**: freeze 50–80 ms on a big impact. It makes hits land.

## 4. Readable tension

- **Show danger before it kills.** Every fail state has a ramp the player can see: color shifting, a face reacting, shaking, a meter filling, a warning sound speeding up. The best tell is a character's face: wince, sweat, panic.
- **Show near-misses.** "SO CLOSE!" or a slow-mo on a narrow escape makes players feel skilled and want another try.
- **Always show why you failed.** Freeze or slow the moment, point at the cause, then play the fail animation. An unclear death feels unfair, and unfair means swipe.
- **Fair randomness**: random within designed bounds, never an unwinnable situation, and never two nasty surprises in a row.

## 5. The loop: short rounds, instant retry, constant novelty

- **Round length**: a first-time player's first round ends in 20–45 s; a good player's lasts 1–3 min. Sessions are a few rounds, not one long one.
- **Escalate fast.** Every ~10–15 s something gets faster, narrower or new. A new element (obstacle, shape, enemy, rule) every 15–30 s keeps people watching *and* players playing.
- **Failure is the punchline.** Make the fail animation funny, gross, spectacular or satisfying, 1–2 s long. It's the most-shared moment and the game-over card.
- **One-tap retry.** The game-over screen's tap target is the whole screen (minus a small separate button like SCORES), and retry starts play instantly.
- **Progress pull**: show the score big, the best, and the gap ("3 MORE TO BEAT YOUR BEST"). Flash "NEW BEST!" with a burst.

## 6. Cards: three screens strangers see

The platform turns three screens into feed cards, all shown live but untouchable, with the title and creator over the bottom ~70 pt and buttons down the right ~56 pt.
1. **Title card**: the ad (§1). Character, scene, hook, a tempting looping moment, maybe a tagline. No instructions.
2. **Game-over card**: what you see swiping back to a game you just played. Show the fail moment's aftermath (the mess, the reaction), the score big and BEST ("NEW BEST!" or "3 MORE TO BEAT IT"). It's a brag and a dare, not a menu: no "TAP TO RETRY" (the platform handles tapping, and a tap does retry). It should make a stranger want to try.
3. **Replay card**: your best run (or a friend's) playing back. It must be watchable with no input: make the state obvious from the visuals alone (score counter, danger tells, events with clear animations).

## 7. Phone ergonomics

- One thumb, portrait. Tap targets ≥ 44 pt (≈ 44 logical units at 360 wide). Primary actions in the lower-middle, never under the card overlays (bottom ~70 pt, right ~56 pt) or the pause pill (top-left).
- Don't make players cover what they need to see: put the action above the thumb, or let a tap anywhere count.
- **Motion controls**:
  - Calibrate relative to how the phone is held when play starts (and after a pause), so it works standing, slumped or in bed.
  - Keep the needed range within ~45° of rotation so the screen stays readable, amplifying in the game if the action needs more (`SKILL.md` §Platform facts).
  - Smooth the reading (~50–100 ms) with a small dead zone near neutral.
  - Show a live on-screen mirror of the reading at all times (a tilting object, a needle or bubble level).
  - Teach the direction with an animated phone icon and arrow.

## 8. Audio design

Many players have sound off, so it never carries the game. For those who have it on, it's half the juice. Make it good:
- **Every action has a sound**, and the important ones are distinct at a glance-equivalent: a success, a near-miss, a fail and a milestone should be recognizable with eyes closed.
- **Vary repeats**: the same sample 10 times a second machine-guns. Vary pitch ±3–5% per play, alternate 2–3 variants, and let streaks climb a scale (each consecutive success a step up).
- **Sounds in key with the music**: pitch melodic effects (pickups, chimes, combos) to notes of the music's scale so they sit inside it rather than clash.
- **Music**:
  - A short loop (8–16 bars) with a clear identity in the first bar.
  - Built in **stems** (base + tension layer, generated together by `Song.stems` and saved with `save_stems` so they keep their balance) so it can intensify by raising a layer's volume, not just by pitching the whole loop up, which sounds cheap past ~5%.
  - Start it with play; on the title card keep only a quiet bed or none.
- **Mix**:
  - Effects louder than music; music about −6 dB under effects (`save_ogg(kind="music")` does this).
  - Duck the music briefly for big moments (a fail, a boss) and let a short stinger play.
  - Stop or filter the music on the game-over screen so the punchline lands.
- **Tone**: plain full-volume square waves are harsh. Use the presets (filtered leads, plucks, triangle bass, detuned pads), short envelopes for effects, and a touch of reverb on big sounds only.
- **Silence is a tool**: a beat of quiet before a payoff (the freeze before an explosion) makes it hit harder.
- **Haptics follow the same beats** as the sound: a tick per small event, a thud on impact, rising pulses with tension.

## 9. Personality

- **A character with a face beats an abstract shape.** Faces react to everything: delight on success, worry near danger, horror on failure.
- **Humor** lands through animation timing (anticipation, overreaction, a beat of stillness before the payoff), not text.
- **Cohesive look**: a limited palette (8–16 colors), consistent pixel size, thick dark outlines for readability on a small screen, strong silhouette contrast against the background.
- **Music**: a short, catchy loop (8–16 s) that can intensify (layer or pitch up with tension). Keep effects louder than music.

## Checklist

**First second**
- [ ] A still of the title card alone tells a stranger what the game is about.
- [ ] Something on the title card is always moving.
- [ ] The title card gives no instructions (no BLOW/TILT/SHAKE/TAP, no phone icons or arrows): it sells the game with stakes, character and a tempting looping moment.
- [ ] A tap anywhere (not under overlays) starts play immediately; the first tap can be the first move.
- [ ] The first seconds of play show how to play with animated affordances, right where the action works.

**First ten seconds**
- [ ] Every input gets a visible response the same frame, plus sound and (in play) haptics.
- [ ] Hints show where and which way, persist until performed, and return if the player stalls.
- [ ] The opening seconds can't be failed, and there's an early win with a reward burst.
- [ ] No text block over 5 words; key text is ≥ 16 px with a shadow or panel.

**Tension and failure**
- [ ] Every fail state has a visible ramp (face, color, shake, meter) before it triggers.
- [ ] The cause of failure is shown clearly; the fail animation is a 1–2 s payoff.
- [ ] Difficulty escalates every ~10–15 s, with something new every 15–30 s; the bot confirms a first-timer's round lasts 20–45 s.

**Loop and cards**
- [ ] Score is big and pops on change; best is saved and "NEW BEST!" celebrates.
- [ ] Retry is one tap; the game-over screen is a good card (aftermath, score, best), with no instructions.
- [ ] A replay is understandable with no input.
- [ ] Nothing important is under the card overlays (bottom ~70 pt, right ~56 pt) or the pause pill.

**Platform**
- [ ] `sys::round` reports Idle / Playing / Over; the score is submitted before Over.
- [ ] Works fully with sound off; haptics never carry information alone.
- [ ] Music loops seamlessly, intensifies with layers, starts with play and gets out of the way on game-over.
- [ ] Effects are varied, in key, louder than the music, and no sound is harsh or clips.
- [ ] The first frame looks good before any asset has loaded.
