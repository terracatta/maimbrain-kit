#!/bin/sh
# Bright Spark's generated art and sound: every asset the game ships, as the
# `mb` command that made it. Restyle the kit by editing art/style.toml (its
# style, palette and [music]/[sfx] words go into every prompt below) and the
# prompts here, then run:
#
#   sh tools/regen.sh                    # everything (~$0.35: 6 images, 1 music call, 16 effects)
#   sh tools/regen.sh host stage         # just these
#   MB="cargo run -q -p mb-cli --" sh tools/regen.sh music   # mb from a monorepo checkout
#
# A new host: Watt's sheet is the style reference for everything else
# (`references` in art/style.toml). For a new character, set
# `references = []`, run `sh tools/regen.sh host`, look at
# art/preview/host.png until it's right (`mb regen assets/host.png --new-seed`),
# put the reference back, then run the rest. Keep the seven frames in this
# order and the 320×352 cells: src/art.rs maps moods to cells and centres the
# glass at HOST_GLASS (measure your new sheet: the glass's centre and radius
# in a cell's pixels). Keep ten emblems in look::category's order; if the
# marquee's panel moves, update MARQUEE_PANEL in src/art.rs.
#
# One asset again with the same recipe: `mb regen assets/burst.png` (`--new-seed`
# for another take, `--reprocess` to redo only the post-processing, free).
# ElevenLabs has no seeds: every effect run is a new take, so read the numbers
# and run the skill's check_audio.py before keeping one. After a new music or
# tension loop, re-derive TENSION_PITCH in src/sound.rs from the two loops'
# lengths (DESIGN.md "Look and sound").

set -e
GAME="$(cd "$(dirname "$0")/.." && pwd)"
MB="${MB:-mb}"
ONLY="$*"
want() { [ -z "$ONLY" ] && return 0; case " $ONLY " in *" $1 "*) return 0;; esac; return 1; }
sfx() { name="$1"; shift; if want "$name"; then $MB sfx "$GAME" "$name" "$@"; fi; }

# ---- Watt: seven expression frames in one sheet (idle, think, nervous,
# happy, star-eyed, flicker, cracked), 4 columns of 320×352 cells.
if want host; then
  $MB art frames "$GAME" host "Watt, a cheerful cartoon lightbulb quiz-show host: a round cream-yellow glass bulb head with a big expressive face (large oval eyes with pupils, eyebrows, a wide mouth), a ribbed silver screw base as his body, a tomato-red bow tie with white polka dots at the collar, two small white-gloved cartoon hands. Seven expression frames, in order: 1 idle, friendly closed-mouth smile, hands at his sides; 2 thinking, one eyebrow raised, eyes glancing up and to the side, one gloved finger on his chin; 3 nervous, very wide eyes, wobbly zigzag mouth, sweat drops flying off the glass, both hands clutching his bow tie; 4 happy, eyes squeezed into happy arcs, huge open grin, both arms thrown up; 5 star-struck, eyes are big mustard five-point stars, enormous grin, jazz hands, small sparkles around the bulb; 6 flickering, the glass dimmed to grey, wincing with eyes squeezed shut, a little zigzag spark at the top; 7 blown out, the glass dark grey and cracked with jagged crack lines, eyes are X marks, mouth a wavy line, a small wisp of smoke from the top, slumped" \
    --frames 7 --cols 4 --size 320x352 --res 2K --seed 1872065215
fi

# ---- The stage behind everything (cover-fitted to the screen).
if want stage; then
  $MB art background "$GAME" stage "a 1950s television quiz-show stage, seen straight on: heavy tomato-red theater curtains with a scalloped valance across the top and drawn back in folds at both sides, between them a deep teal back wall shaded with a coarse halftone dot pattern, a few big mustard and cream atomic-age starbursts and boomerang shapes high on the wall, a curved mustard stage edge at the very bottom; the middle of the wall is plain calm teal with nothing in it" \
    --colors 24 --max-kb 150 --seed 38682749
fi

# ---- Ten category emblems (look::category order), 5 columns of 112×112 cells.
if want cats; then
  $MB art frames "$GAME" cats "ten quiz-category emblems, each a single bold object inside its own round cream badge with a thick ink rim, in order: 1 science, a bubbling chemistry flask; 2 space, a ringed planet with a little star; 3 world, a globe; 4 nature, a leafy sprig; 5 food, a slice of pie; 6 numbers, a stack of three mismatched building blocks; 7 words, an open book; 8 inventions, a gear with a spark; 9 body, a heart; 10 time, an alarm clock. Every badge the same size and shape; only the object inside differs" \
    --frames 10 --cols 5 --size 112x112 --res 2K --seed 461955705
fi

# ---- The title's marquee sign (the title is lettered on it in code) and the
# starburst (the streak badge, the golden card).
if want marquee; then
  $MB art ui "$GAME" marquee "a blank 1950s theater marquee sign: a wide rounded tomato-red sign board with a thick ink outline, its border studded with a single row of round cream light bulbs evenly spaced all the way around, a large empty flat teal panel in the middle for lettering, a small mustard atomic starburst on each end" --size 680x300 --seed 1001529302
fi
if want burst; then
  $MB art ui "$GAME" burst "a bold mustard-yellow starburst with twelve sharp points and a thick ink outline, a cream inner starburst on top of it, flat with a little halftone shading" --size 256x256 --seed 1342436984
fi

# ---- The icon.
if want icon; then
  $MB art icon "$GAME" "Watt the cartoon lightbulb quiz host (the character in the reference) with star-shaped mustard eyes and a huge grin, his face and red polka-dot bow tie filling most of the frame, a big mustard starburst behind him on a tomato-red ground with halftone dots" --seed 723433595
fi

# ---- Music: a big-band swing loop split into a calm base and a bright high
# layer (src/sound.rs keeps the high layer at half and opens it with the
# streak and the clock), and a ticking stopwatch loop faded in as the clock
# runs down.
if want music; then
  $MB music "$GAME" music "an upbeat swinging game-show theme: punchy brass section riffs and stabs, walking upright bass, brushed swing drums with ride cymbal, bouncy piano comping, confident and fun, steady groove all the way through" \
    --bars 8 --bpm 160 --layers --seed 1750449807
fi
sfx tension "a steady tense ticking stopwatch, sharp even mechanical ticks, nothing else" --seconds 3 --loop

# ---- Effects (the [sfx] words in art/style.toml are added to each).
sfx start "a short big-band brass fanfare sting announcing the start of a quiz round, trumpets and trombones in unison, bright and confident" --seconds 1.5
sfx flip "a crisp stiff cardboard quiz card flipping over, one quick paper flick" --seconds 0.4
sfx tap "a satisfying chunky push-button click on a vintage game-show contestant panel" --seconds 0.25
sfx right "a bright bell ding together with a short punchy big-band brass stab, a correct answer" --seconds 0.9
sfx wrong "a harsh vintage game-show buzzer, a wrong answer, one short blast" --seconds 0.7
sfx timeup "an old mechanical alarm bell ringing briefly, time is up" --seconds 0.9
sfx tick "a single loud crisp mechanical clock tick" --seconds 0.2
sfx fizzle "a lightbulb flickering, a short electric buzz and crackle" --seconds 0.6
sfx multup "a fast airy whoosh rising into a bright sparkle, a streak bonus" --seconds 0.7
sfx golden "a short triumphant fanfare, big-band trumpets over a cymbal swell with a sparkling chime on top" --seconds 1.8
sfx heart "a warm quick rising harp glissando ending on a soft bell chime, an extra life" --seconds 0.9
sfx pop "a glass lightbulb bursting with a sharp pop and tinkling glass shards, then an electrical fizzle and sizzle dying out" --seconds 1.8
sfx over "a comic sad descending muted trombone phrase, four notes, game-show loss" --seconds 2.2
sfx best "a celebratory big-band brass fanfare hit, warm trumpets and trombones holding a bright major chord, a new high score" --seconds 2.0
sfx ui "a soft clicky toggle switch on a vintage radio" --seconds 0.2

$MB art check "$GAME"
