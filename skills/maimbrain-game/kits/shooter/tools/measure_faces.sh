#!/usr/bin/env bash
# Finds the dark face window in each sprite of art/sprites/ (the biggest dark,
# opaque blob; one per frame for the swooper sheet) and prints its center relative to the sprite's center, in
# logical units at the size src/draw.rs draws it (2 texels per unit). Copy the
# numbers into the offsets at the top of src/draw.rs after regenerating art.
#
#   tools/measure_faces.sh            pip jelly darter bulb spinner boss swooper
#   tools/measure_faces.sh boss 16    one sprite, a looser darkness threshold (%)
#
# Needs ImageMagick (`magick`).
set -euo pipefail
G="$(cd "$(dirname "$0")/.." && pwd)"
names=${1:-"pip jelly darter bulb spinner boss swooper"}
th=${2:-12}
for n in $names; do
  f="$G/art/sprites/$n.png"
  [ -f "$f" ] || { echo "$n: no $f (tools/regen.sh atlas restores it)"; continue; }
  read -r W H <<< "$(magick identify -format '%w %h' "$f")"
  echo "$n (${W}x${H} texels, drawn $((W / 2))x$((H / 2)) units):"
  magick "$f" -alpha extract \( "$f" -alpha off -colorspace gray -threshold "$th%" -negate \) -compose multiply -composite -threshold 50% \
    -define connected-components:verbose=true -define connected-components:area-threshold=150 -connected-components 8 null: 2>/dev/null |
    awk -v W="$W" -v H="$H" '/srgb\(255,255,255\)|gray\(255\)/ { split($3, c, ","); split($2, g, /[x+]/);
      if ($4 + 0 > 4000 && g[1] > W * 0.8) next;  # the whole body, not a window
      printf "%08d  window %sx%s texels, center (%+.1f, %+.1f) units from the sprite center\n", $4, g[1], g[2], (c[1] - W / 2) / 2, (c[2] - H / 2) / 2 }' |
    sort -rn | head -"$([ "$n" = swooper ] && echo 4 || echo 1)" | cut -c9-
done
echo "(a frame sheet lists one window per frame: subtract each frame's cell origin)"
