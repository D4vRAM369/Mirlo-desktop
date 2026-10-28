#!/usr/bin/env bash
# Sube el yt-dlp fijado a <versión>, sube el parche de Mirlo (0.1.0 → 0.1.1) y añade la entrada al CHANGELOG.
# Lo usa el workflow yt-dlp-update.yml; también sirve a mano: `scripts/bump-yt-dlp.sh 2026.11.03`.
set -euo pipefail
ytdlp=$1
old=$(grep -oP '^  "version": "\K[^"]+' package.json)
new=$(awk -F. '{ print $1 "." $2 "." $3 + 1 }' <<<"$old")
today=$(date -u +%F)

sed -i "s/^const VERSION = '.*'/const VERSION = '$ytdlp'/" scripts/fetch-yt-dlp.mjs
sed -i "0,/\"version\": \"$old\"/s//\"version\": \"$new\"/" package.json src-tauri/tauri.conf.json
sed -i "0,/^version = \"$old\"/s//version = \"$new\"/" src-tauri/Cargo.toml
sed -i "/^name = \"mirlo\"$/{n;s/\"$old\"/\"$new\"/}" src-tauri/Cargo.lock
# package-lock.json repite la versión en la raíz y en packages[""]: las dos primeras apariciones
sed -i "0,/\"version\": \"$old\"/s//\"version\": \"$new\"/; 0,/\"version\": \"$old\"/s//\"version\": \"$new\"/" package-lock.json

# entrada nueva justo encima de la primera "## [" del CHANGELOG
entry="## [$new] — $today\n\n- yt-dlp actualizado a $ytdlp. · *yt-dlp updated to $ytdlp.*\n"
sed -i "0,/^## \[/s//$entry\n## [/" CHANGELOG.md

echo "$old → $new · yt-dlp $ytdlp"
