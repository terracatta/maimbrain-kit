#!/bin/sh
# Remakes Cake Keep's generated art and sound from art/style.toml and the
# descriptions below. To restyle the kit: edit art/style.toml (the look, the
# palette, the [music] and [sfx] words), edit the descriptions here for your
# own characters, then run this. Everything else (sizes, frames) stays put,
# so src/art.rs keeps working.
#
#   tools/regen.sh            # everything (about $1: 17 images, 2 loops, 17 effects)
#   tools/regen.sh art        # images only      tools/regen.sh sound   # audio only
#   tools/regen.sh tower_pop  # one asset (any name below)
#
# Needs `mb keys set gemini` (images, music) and `mb keys set elevenlabs`
# (effects); see docs/GENERATE.md. MB=... overrides the mb command;
# FLAGS=--dry-run shows every prompt and price without calling anything.
# Sizes: sprites are quantized to 96 colours (art/style.toml `colors`), the
# map to fit 200 KB, music at 32 kHz, so the kit stays under ~1 MB.
# Do the hero (jelly_gummy) first and look at art/preview/jelly_gummy.png:
# art/style.toml lists it under `references`, so everything after matches it.
# After regenerating, look at every art/preview/*.png, then check in the game:
#   - tower sheets: mb's frame splitter can put a tall level-3 model first;
#     fix TOWER_LEVEL_FRAME in src/art.rs to match the preview.
#   - sizes on the board: JELLY_CELL_PER_R (src/art.rs), TOWER_LEVEL_GROW (src/draw.rs).
#   - the map: its road must sit on PATH. map_sketch.py draws the layout it follows.
set -e
cd "$(dirname "$0")/.."
G=.
MB=${MB:-mb}
FLAGS=${FLAGS:-}
want() { [ -z "$ONLY" ] || [ "$ONLY" = "$1" ] || [ "$ONLY" = "$2" ]; }
case "$1" in "" | all) ONLY= ;; *) ONLY=$1 ;; esac

frames() { # name frames cell "description"
  if want "$1" art; then $MB art frames $FLAGS $G "$1" "$4" --frames "$2" --cols "$2" --size "$3" --out "assets/$1.png"; fi
}
sprite() { # name size "description"
  if want "$1" art; then $MB art sprite $FLAGS $G "$1" "$3" --size "$2" --out "assets/$1.png"; fi
}
sfx() { # name seconds "description" (ElevenLabs allows 4 at once: keep these sequential)
  if want "$1" sound; then $MB sfx $FLAGS $G "$1" "$3" --seconds "$2" --quality 3; fi
}

# ---- The hero first: everything else matches it ------------------------
frames jelly_gummy 5 96x96 "a round squishy translucent red-pink jelly gumdrop blob creature with an angry face: furious slanted eyebrows, glaring eyes, a grumpy frown, two stubby feet. Frames left to right: waddling with the left foot forward, waddling with the right foot forward, squashed flat and wide from being hit, bursting apart into a splat of jelly drops, and finally the same blob looking smug and happy with closed smiling eyes and a full belly"

# ---- Jellies: walk A, walk B, pop (src/art.rs: JELLY + CreepKind) ------
frames jelly_zipper 3 80x80 "a small slim lemon-yellow jelly blob shaped like a teardrop, leaning forward as if sprinting, a determined angry face with narrowed eyes and gritted teeth, tiny quick legs. Frames left to right: running stride with the left leg forward, running stride with the right leg forward, bursting apart into a splat of yellow jelly drops"
frames jelly_helmet 3 96x96 "a sturdy round grape-purple jelly blob wearing a dented grey clay soldier's helmet pulled low over furious eyes, a grim frown, stubby feet. Frames left to right: marching with the left foot forward, marching with the right foot forward, bursting apart into a splat of purple jelly drops with the helmet flying off"
frames jelly_splitter 3 112x112 "a big chubby tangerine-orange jelly blob with a deep crease running down the middle of its body, an angry grumpy face, stubby feet. Frames left to right: waddling with the left foot forward, waddling with the right foot forward, splitting apart down the crease into three tiny orange jelly blobs flying outward"
frames jelly_mini 3 64x64 "a tiny tangerine-orange baby jelly blob with big angry eyebrows and a pouting frown, very short stubby feet. Frames left to right: hopping with the left foot forward, hopping with the right foot forward, bursting apart into a small splat of orange jelly drops"
frames jelly_king 3 144x144 "a huge magenta-pink jelly blob king wearing a chunky golden clay crown with a red gem, a tiny purple cape, a furious royal scowl with bushy eyebrows, thick stomping feet. Frames left to right: stomping with the left foot forward, stomping with the right foot forward, bursting apart into a big splat of pink jelly drops with the crown tumbling"

# ---- Towers: levels 1, 2, 3 (TOWER_LEVEL_FRAME in src/art.rs) ----------
frames tower_pop 3 128x128 "a gumball shooter tower: a round clear candy-glass dome packed with colorful clay gumballs (pink, yellow, blue, white) on a chunky bubblegum-pink clay pedestal, a short stubby round nozzle sticking out the front. The three frames left to right are its upgrade levels 1, 2 and 3, each clearly bigger and fancier: level 1 small and simple; level 2 taller with a gold band around the pedestal and two nozzles; level 3 grand, a bigger dome, three nozzles, gold trim and a gold star on top. Smallest on the left, biggest on the right"
frames tower_chill 3 128x128 "an ice-cream cone tower: a crunchy waffle cone standing point-down in a lavender clay stand, topped with scoops of icy pale-blue ice cream with frosty white sparkles, a sleepy happy little face on the top scoop. The three frames left to right are its upgrade levels 1, 2 and 3, each clearly bigger and fancier: level 1 one blue scoop; level 2 two scoops, blue and white, with little icicles dripping; level 3 three scoops, blue, white and pink, with a crown of ice crystals. Smallest on the left, biggest on the right"
frames tower_boom 3 128x128 "a cherry bomb launcher tower: a squat round red clay mortar cannon pointing up on a chocolate-brown clay base, a big glossy dark-red cherry bomb with a brown stem fuse peeking out of its mouth. The three frames left to right are its upgrade levels 1, 2 and 3, each clearly bigger and fancier: level 1 small and plain; level 2 wider with two gold bands and a spare cherry bomb beside it; level 3 a big heavy mortar with three gold bands, gold rivets and a little pile of cherry bombs"

# ---- The cake: happy, nervous, panic, wince, the empty plate ------------
frames cake 5 176x176 "a round two-layer strawberry sponge cake with pink frosting drips, rainbow sprinkles and a red cherry on top, sitting on a white plate, with a cute face on the front of the sponge. Frames left to right: 1 happy, a content smile and rosy cheeks; 2 nervous, worried raised eyebrows and a wobbly flat mouth; 3 panicking, wide eyes, an open screaming mouth and a sweat drop; 4 wincing, eyes squeezed shut and teeth clenched as if just bitten; 5 the same white plate now empty except for a few crumbs and a lonely cherry with X eyes"

# ---- Props ---------------------------------------------------------------
sprite pad 112x112 "a flat square build pad seen straight from above: a thick soft-cornered square shortbread biscuit made of golden-cream clay with a pressed criss-cross pattern and a raised rim, very plain in the middle"
sprite bomb 48x48 "a small glossy round dark-red cherry bomb with a short brown stem as its fuse and a bright yellow spark at the tip"

# ---- The map: drawn over the layout sketch so the road matches PATH ------
if want map art; then
  python3 tools/map_sketch.py art/ref/map_sketch.png
  $MB art background $FLAGS $G map "a top-down candy garden diorama sculpted from plasticine: soft mint-green clay lawn with thumbprint texture, and a winding cream-colored clay road with slightly raised caramel-colored edges that follows the light road in the layout sketch exactly: the same position, the same width, the same right-angle turns, entering at the top and ending at a small round clearing near the bottom. Beside the road keep wide stretches of plain open lawn. Only small, low decorations, mostly near the screen edges and the bottom: tiny clay candy flowers, little gumdrops, a few lollipop bushes, sprinkles in the grass. No characters, no buildings, no towers, no cake" --ref art/ref/map_sketch.png --size 540x960 --colors 256 --max-kb 200
fi

# ---- The icon ------------------------------------------------------------
if want icon art; then
  $MB art icon $FLAGS $G "a clay strawberry cake with pink frosting and a cherry on top sitting on a plate, its face nervous and wide-eyed, while two angry red jelly blobs creep up from the bottom corners, on a mint-green clay lawn, bold and simple"
fi

# ---- Music: a march and its urgent twin at the same tempo. src/sound.rs
# crosses from one to the other on boss waves and the last heart. ----------
if want music_base sound; then $MB music $FLAGS $G music_base "a bouncy cheerful circus marching band loop: oompah tuba bass on every beat, crisp snare and bass drum march pattern, a cheeky clarinet and cornet melody, bright glockenspiel accents, playful and confident" --bpm 120 --bars 8 --rate 32000 --quality 2; fi
if want music_tension sound; then $MB music $FLAGS $G music_tension "an urgent frantic circus marching band for when the big boss arrives: driving snare drum rolls, pounding bass drum and tuba, blaring trumpet and trombone stabs, rising clarinet runs, cymbal crashes, tense and exciting" --bpm 120 --bars 8 --rate 32000 --quality 2; fi

# ---- Effects (names match src/sound.rs FILES) -----------------------------
sfx start 0.8 "a cheerful circus whistle toot with a snare drum flam, let's go"
sfx build 0.5 "a squishy plasticine thunk, a lump of clay pressed down onto a wooden table, with a soft pop"
sfx upgrade 0.8 "a bright rising glockenspiel sparkle over a squishy clay squeeze"
sfx pop 0.5 "a quick gumball thwip, a small rubbery candy ball shot out of a tube"
sfx lob 0.5 "a hollow cartoon mortar thoomp launching a cherry into the air"
sfx frost 0.7 "a frosty icy shimmer, glassy sparkling chimes with a cold airy whoosh"
sfx boom 0.8 "a cartoon cherry bomb going off, a punchy small boom with a wet squishy splat"
sfx hit 0.5 "a tiny soft rubbery tap of a gumball bouncing off jelly"
sfx splat 0.5 "a squelchy wet jelly pop, a gummy blob bursting with a juicy splat"
sfx coin 0.5 "a single bright little coin clink"
sfx bite 0.5 "a cartoon chomp, one big bite into soft sponge cake"
sfx wave 1.2 "a short cheeky marching band bugle call, two rising notes on a cornet"
sfx boss 1.8 "a low dramatic tuba and trombone fanfare over a timpani roll, the big boss arrives"
sfx deny 0.5 "a short rubbery clown horn honk, a cheeky no"
sfx ui 0.5 "a soft squishy clay button press, very short"
sfx over 2.2 "a huge cartoon chomp-down and gulp, then a sad descending trombone wah wah wah"
sfx best 1.5 "a short triumphant circus band fanfare with a cymbal crash, hooray"
