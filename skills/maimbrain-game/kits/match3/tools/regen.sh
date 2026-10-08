#!/bin/sh
# Regenerates Poppets' art and sound from the sidecars (assets/*.gen.json,
# icon.png.gen.json) under the current art/style.toml. Every image and sound
# keeps its recorded prompt, seed, size and budget; only the style changes.
#
#   tools/regen.sh hero    the coral poppet only (look at it, then make it the style reference)
#   tools/regen.sh art     the rest of the cast, the atlas, the window, the paper, the icon
#   tools/regen.sh sound   the lullaby, the tension drone and every effect
#   tools/regen.sh all     art + sound
#
# Restyling the whole kit (DESIGN.md, "Make it yours" step 1):
#   1. Edit art/style.toml (style, keywords, palette, [music], [sfx]) and set
#      `references = []`, so the old hero doesn't pull the new art back.
#   2. tools/regen.sh hero, then open art/preview/p_coral.png. Again with
#      `mb regen assets/p_coral.png --new-seed` (or `--prompt "…"` for a new
#      character) until it's right.
#   3. cp assets/p_coral.png art/ref/poppet.png and set
#      `references = ["art/ref/poppet.png"]` again.
#   4. tools/regen.sh all, look at every art/preview/*.png, regenerate what's off.
#   5. The sounds are tuned in src/sound.rs: TENSION_PITCH puts the drone in the
#      lullaby's key; check both with a chroma tool or by ear.
#
# To change what a sprite IS (a new cast), pass a new description:
#   mb regen assets/p_orange.png --prompt "a plump square toaster creature…"
# The sprite PNGs are deleted after packing (they'd ship twice); their
# sidecars stay, so `mb regen assets/<name>.png` still works.
#
# MB overrides the command (in the Maimbrain repo:
#   MB="mise exec -- cargo run -q -p mb-cli --" tools/regen.sh all).
# Costs (Oct 2026): ~$0.03 per image (~$0.40 for all art), $0.04 per music
# loop, ~$0.002 per effect. ElevenLabs allows 4 requests at once, so effects
# run one after another.
set -e
cd "$(dirname "$0")/.."
G="$(pwd)"
MB="${MB:-mb}"
mb() { $MB "$@"; }

CAST="p_coral p_orange p_yellow p_green p_blue p_purple p_pink rock rainbow"
SFX="start select swap bounce pop chime line bomb rainbow made rock level shuffle drowsy phew over best ui thud"

hero() {
    mb regen "$G/assets/p_coral.png"
}

art() {
    for s in $CAST; do
        [ "$s" = p_coral ] && [ -f "$G/assets/p_coral.png" ] && continue
        mb regen "$G/assets/$s.png"
    done
    paths=""
    for s in $CAST; do paths="$paths $G/assets/$s.png"; done
    # Same order as before, so src/cast.rs keeps its names.
    mb art atlas "$G" cast $paths
    for s in $CAST; do rm -f "$G/assets/$s.png"; done
    mb regen "$G/assets/bg.png"
    mb regen "$G/assets/paper.png"
    mb regen "$G/icon.png"
}

sound() {
    mb regen "$G/assets/music.ogg"
    mb regen "$G/assets/tension.ogg"
    for s in $SFX; do mb regen "$G/assets/$s.ogg"; done
}

case "${1:-}" in
    hero) hero ;;
    art) art ;;
    sound) sound ;;
    all) art; sound ;;
    *) sed -n '2,9p' "$0"; exit 1 ;;
esac
mb art check "$G"
