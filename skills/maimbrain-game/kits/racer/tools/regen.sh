#!/bin/sh
# Swerve's generated sound (and icon): every asset the game ships, as the
# `mb` command that made it. Restyle the kit by editing art/style.toml (the
# [music] and [sfx] words are added to every prompt below) and the prompts
# here, then run:
#
#   sh tools/regen.sh                  # everything (~$0.12: one music call, 18 effects, the icon)
#   sh tools/regen.sh lane coin crash  # just these
#   MB="cargo run -q -p mb-cli --" sh tools/regen.sh music   # mb from a monorepo checkout
#
# One asset again with the same recipe: `mb regen assets/coin.ogg` (`--new-seed`
# for another take). Lyria doesn't honor keys or exact tempos: after a new
# music track, read the BPM `mb music` prints and set `MUSIC_ROOT` in
# src/sound.rs to the track's key so the coin chimes stay in tune (see DESIGN.md
# "Look and sound"). ElevenLabs has no seeds: every run is a new take, so listen
# (or at least read the numbers and run the skill's check_audio.py) before keeping one.
#
# The 3D art is drawn in code (src/scene.rs, colors in src/look.rs); this file
# doesn't touch it. The old procedural set is tools/gen_audio.py (free, offline).

set -e
GAME="$(cd "$(dirname "$0")/.." && pwd)"
MB="${MB:-mb}"
want() { [ -z "$ONLY" ] && return 0; case " $ONLY " in *" $1 "*) return 0;; esac; return 1; }
ONLY="$*"

sfx() { name="$1"; shift; if want "$name"; then $MB sfx "$GAME" "$name" "$@"; fi; }

# ---- Music: one track, split into a calm low layer and a bright high layer
# the game opens up with speed, boosts and near-miss chains.
if want music; then
  $MB music "$GAME" music "uptempo outrun synthwave in A minor, 120 BPM, for a sunset highway chase: punchy gated drum machine (kick on every beat, huge gated snare on 2 and 4, sixteenth hi-hats), a driving octave-jumping eighth-note analog bass, warm wide polysynth pad chords, a bright arpeggiator and a catchy heroic lead hook; constant full energy" \
    --bpm 120 --bars 8 --layers --seed 23
fi

# ---- Loops the game pitches and fades with speed.
sfx engine "a sporty four-cylinder car engine at steady cruising revs heard from just behind the car, smooth even hum with a slight growl, constant pitch" --seconds 2 --loop
sfx wind "steady rushing wind noise of driving fast with the windows down, broadband air roar, constant, no gusts" --seconds 3 --loop

# ---- Driving.
sfx start "a sports car engine revving up quickly from idle to high revs, rising pitch, then the tyres bite and it launches" --seconds 2
sfx lane "an instant sharp tyre swish: a bright hissing rubber-on-asphalt swoosh as a car snaps into the next lane, starts immediately, short airy whoosh" --seconds 0.5
sfx squeal "a short sharp tyre squeal on asphalt, a quick chirp" --seconds 0.6
sfx pass "a single car whooshing past very close at high speed, doppler pass-by, quick rise and fall" --seconds 1
sfx near "a very close high-speed near miss: a sharp air whoosh past the window with a bright metallic ting at the end" --seconds 0.9
sfx coin "a soft warm coin pickup: one gentle rounded synth pluck, mellow marimba-like tone with a soft attack, no sparkle, no shimmer, no high ring, one short note" --seconds 0.5
sfx boost "a turbo boost ignition: a whump of flame then a rising jet-engine roar" --seconds 1.6
sfx smash "a car ramming through a traffic barrier at speed: a hard metal crunch with plastic shattering" --seconds 0.8
sfx blinker "a single car turn signal relay click, dry plastic tick" --seconds 0.5
sfx alert "a car dashboard warning chime: two quick high bright electronic beeps, ding-ding, clean synth tone" --seconds 0.6
sfx oil "a car hitting an oil slick: a wet splat then tyres skidding and sliding" --seconds 0.9
sfx speedup "a short rising synth sweep with a bright whoosh, a gear shift up" --seconds 0.6

# ---- The end of a round, and the UI.
sfx crash "a huge car crash at speed: heavy metal impact and crunch, the car rolling, a big burst of shattering glass, debris tinkling down" --seconds 2.6
sfx over "a short sad retro synth sting: three descending notes on a warm analog synth, ending low" --seconds 2
sfx best "arcade game high score jingle: a quick bright ascending synth arpeggio, do-mi-sol-do, ending on a sparkling sustained major chord, retro analog synth, celebratory" --seconds 2
sfx ui "a soft short UI click, a muted synth blip" --seconds 0.5

# ---- The icon (the in-game art is code; see art/style.toml [icon]).
if want icon; then
  $MB art icon "$GAME" "a chunky low-poly teal coupe with a white racing stripe and a big black spoiler, seen from low behind, racing down a three-lane desert highway toward a huge low sunset sun between red mesas, glowing red tail lights, blue exhaust flames, palm trees whipping past"
fi
