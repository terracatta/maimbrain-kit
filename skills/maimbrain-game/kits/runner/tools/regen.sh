#!/bin/sh
# Regenerates every generated asset of this game: the art from art/style.toml,
# the music and the sound effects. Edit the style guide (and the subjects
# below) to restyle the game, then run:
#
#   sh tools/regen.sh            # everything (~$1: ~16 images, 1 track, 13 effects + 4 synthesized)
#   sh tools/regen.sh art        # images only (then the atlas and the icon)
#   sh tools/regen.sh music      # the music only
#   sh tools/regen.sh sfx        # the sound effects only
#
# Needs `mb` (or set MB="cargo run -q -p mb-cli --" in the Maimbrain repo) and
# keys: `mb keys set gemini` (images, music) and `mb keys set elevenlabs`
# (effects). Every command prints its cost first. Look at each preview in
# art/preview/ and redo what's off with `mb regen assets/<name>.png --new-seed`
# (or `--variants 3` and `mb art pick`). Then rebuild the atlas (the last
# step of `art`), since the game draws sprites from assets/sprites.png.
#
# Sizes are art-pixel grids: this kit draws one art pixel 1.5 units wide
# (pixel_scale = 3 texels), so a 44×44 grid is a 66×66-unit sprite.
set -e
cd "$(dirname "$0")/.."
G=.
MB=${MB:-mb}
what=${1:-all}

if [ "$what" = all ] || [ "$what" = art ]; then
  # The hero first: its run cycle becomes the style reference for the rest
  # (art/style.toml lists assets/hero_run.png under references). The rough
  # sheet in art/ref/hero_sketch.jpg pins the character down.
  $MB art frames $G hero_run "the round bread bun hero running fast: a golden-brown bun with sesame seeds, big white eyes with black pupils looking ahead, a determined little smile, pink cheeks, cream underside, short dark legs in red sneakers with white soles. A 4-frame run cycle, one bun per frame, with clearly different leg positions: 1 front leg reaching far forward and back leg pushing off behind, 2 legs passing under the body with the body bobbed up, 3 the opposite leg reaching forward, 4 both feet off the ground mid-stride. The body leans forward slightly" --frames 4 --grid 44x44 --ref art/ref/hero_sketch.jpg
  $MB art frames $G hero_air "the same round bread bun hero (golden-brown bun, sesame seeds, big white eyes, pink cheeks, cream underside, short dark legs, red sneakers) in four separate poses, one bun per frame: 1 jumping upward, legs tucked up under the body, eyes looking up, mouth open in a happy O; 2 falling, legs stretched straight down, eyes wide and worried, small frown; 3 curled into a tight round ball mid-somersault, sneakers tucked against the bun, eyes squeezed shut in happy arcs; 4 bonked and dazed, dizzy X-shaped eyes, wobbly zigzag mouth, legs splayed out" --frames 4 --grid 44x44
  # Obstacles. The crate and log are drawn sliced (edges kept, middles
  # repeated: look.rs CRATE_X/Y, LOG_X/Y), so they need even frames and bark.
  $MB art sprite $G crate "a sturdy wooden crate seen straight from the front: a thick dark-brown wooden frame around all four edges with grey metal corner brackets and rivets; inside, three horizontal light orange-brown planks separated by dark lines, with wood grain streaks and a bright highlight along the top edge of each plank; square" --grid 24x24
  $MB art sprite $G log "a long fallen tree log lying horizontally, seen from the side: rough brown bark with a few knots running the whole length, the left end a pale cut face with growth rings, the right end also cut flat, perfectly straight and even thickness" --grid 64x20
  $MB art frames $G gull "a plump white seagull with grey wings, an orange beak and an angry brow, flying toward the left (facing left), a 4-frame wing flap: wings up, wings level, wings down, wings level" --frames 4 --grid 52x32
  $MB art frames $G skimmer "a sleek dark purple-grey swift bird with long sharp swept-back wings, a white belly and a fierce red eye, flying perfectly level and horizontal, fast and low toward the left (facing left), body horizontal, a 4-frame quick wing beat: wings up, wings level, wings down, wings level" --frames 4 --grid 52x32
  $MB art frames $G bee "a round fuzzy bumblebee with bold yellow and black stripes, an angry face and little stinger, facing left, translucent white wings buzzing, 2 frames: wings up, wings down" --frames 2 --grid 32x30
  $MB art frames $G coin "a shiny gold coin with a raised star in the middle and a darker rim, spinning: 6 frames from face-on to edge-on and back" --frames 6 --grid 16x16
  # Sky and ground.
  $MB art sprite $G sun "a round bright yellow cartoon sun with a soft pale-yellow rim and short chunky rays, a simple happy closed-eye smile" --grid 28x28
  $MB art sprite $G moon "a pale cream crescent moon with two small craters, glowing softly, a sleepy closed-eye face" --grid 22x22
  $MB art frames $G cloud "three different fluffy white cumulus clouds with flat bottoms and a light blue-grey shaded underside, wide and puffy" --frames 3 --grid 48x24
  $MB art tile $G dirt "rich brown soil seen from the side, with small rounded pebbles, a few buried stones and tiny roots, two shades of brown with dark specks" --grid 32x32
  $MB art tile $G grass "dense bright green grass turf seen from the side, short blades and lighter highlight tufts, evenly spread" --grid 16x16
  # Parallax: layer 0 (the sky) only guides the others; the game draws its
  # own banded sky so it can drift day → night, and doesn't ship hills_0.
  # Layers whose shapes cover most of the image border can come back with the
  # background unkeyed (opaque): keep each layer's shapes in the lower part.
  $MB art layers $G hills "a sunny meadow world. The scene from back to front, one part per layer: layer 1 is a bright blue sky in flat horizontal color bands, lighter near the horizon, with a few puffy white pixel clouds; layer 2 is a range of far-off pale lilac and blue mountains with snowy tips; layer 3 is rolling bright green hills dotted with round bushes; layer 4 is a row of round leafy trees, tall grass tufts and small flowers. Draw only the part that belongs to this layer" --count 4 --size 1440x960
  $MB regen assets/hills_2.png --prompt "a sunny meadow world: low rolling bright green hills dotted with round bushes and grass tufts, filling only the bottom quarter of the image, everything above them empty — a middle-distance layer (3 of 4 from the back), drawn in front of far mountains"
  rm -f assets/hills_0.png assets/hills_0.png.gen.json
  # The atlas the game draws from (src/sprites.rs has the rects).
  $MB art atlas $G sprites assets/hero_run.png assets/hero_air.png assets/crate.png assets/log.png assets/gull.png assets/skimmer.png assets/bee.png assets/coin.png assets/sun.png assets/moon.png assets/cloud.png assets/dirt.png assets/grass.png --rust src/sprites.rs
  $MB art icon $G "the bread bun hero in red sneakers leaping joyfully toward the viewer over a wooden crate, a gold coin flying beside it, rolling green hills and a bright blue sky with a puffy cloud behind"
fi

if [ "$what" = all ] || [ "$what" = music ]; then
  # Two stems that loop together: music_base (bass and drums, always on in
  # play) and music_hi (lead and arpeggios, swelling in with speed: sound.rs).
  $MB music $G music "an energetic, joyful platformer stage theme in C major, 150 BPM: a memorable bouncy square-lead hook that repeats every two bars, an octave-jumping bass, quick sparkly arpeggio fills, crisp chip drums with a steady four-on-the-floor kick and snare backbeat, bright and sunny" --layers --bpm 150 --bars 8 --seed 880391115
  rm -f assets/music.ogg assets/music.ogg.gen.json
fi

if [ "$what" = all ] || [ "$what" = sfx ]; then
  # The frequent, tonal ones (coin, arc, milestone, new) are synthesized in
  # the music's key instead, so they stay soft and on pitch: tools/gen_sfx.py
  # (free, offline; edit its notes if the music's key changes).
  python3 tools/gen_sfx.py
  fx() { $MB sfx $G "$1" "$2" --seconds "$3"; }
  fx jump "a bouncy cartoon jump boing, a springy rising pitch, retro console platformer" 0.5
  fx jump2 "a higher, brighter double-jump boing with a quick sparkly spin whoosh" 0.5
  fx land "a soft short cartoon footstep thump, landing on grass" 0.5
  fx close "a fast air whoosh rushing past, a near-miss swoosh" 0.5
  fx faster "a rising speed-up zip, a retro game power-up sweep" 0.8
  fx bonk "a cartoon bonk: a head hitting a wooden crate, a hollow knock with a comic boing" 0.6
  fx fall "a descending slide whistle, a cartoon character falling down a hole" 1.2
  fx bounce "a small cartoon bounce thud with a light boing" 0.5
  fx stuck "a cartoon squelchy pop and thunk, getting stuck head-first in a hole" 0.6
  fx over "a short sad retro game-over jingle, three descending notes, chiptune" 1.6
  fx best "a triumphant short retro victory fanfare jingle, chiptune" 2.0
  fx start "a quick upbeat retro 'ready, go' blip jingle, chiptune" 0.7
  fx ui "a short soft click, a retro menu select tick" 0.5
fi

echo "done: check art/preview/, then python3 tools/check_audio.py assets/*.ogg (or the skill's scripts/check_audio.py), mb art check ., and the mix: tools/mixsim.py"
