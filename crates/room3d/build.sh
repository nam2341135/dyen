#!/usr/bin/env bash
# Build phòng 3D ra crates/web/dist/room/ (vỏ nạp lười ./room/room3d.js). Chạy từ bất kỳ đâu.
#   ./build.sh          bản đo (profile room-release: opt-level 3, LTO, wasm-opt -O3, brotli)
#   ./build.sh dev      bản lặp nhanh (profile room-dev: Bevy tối ưu, code dự án không; không wasm-opt)
# Thường được crates/web/build.sh gọi sau khi tạo lại dist/.
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)
profile=room-release
[ "${1:-}" = dev ] && profile=room-dev
cargo build --manifest-path "$root/Cargo.toml" -p room3d --lib \
  --target wasm32-unknown-unknown --profile "$profile"
out="$root/target/wasm32-unknown-unknown/$profile/room3d.wasm"
dist="$root/crates/web/dist/room"
rm -rf "$dist" && mkdir -p "$dist"
wasm-bindgen --target web --no-typescript --out-name room3d --out-dir "$dist" "$out"
for f in "$here"/assets/*.glb "$here"/assets/*.png; do
  [ -e "$f" ] && cp "$f" "$dist/"
done
size() { stat -c %s "$1"; }
raw="$dist/room3d_bg.wasm"
before=$(size "$raw")
if [ "$profile" = room-release ]; then
  wasm-opt -O3 --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext \
    --enable-mutable-globals --enable-reference-types -o "$dist/opt.wasm" "$raw"
  mv "$dist/opt.wasm" "$raw"
  brotli -q 11 -k -f "$raw" "$dist/room3d.js"
  echo "room3d: wasm $before B → -O3 $(size "$raw") B → brotli $(size "$raw.br") B; js brotli $(size "$dist/room3d.js.br") B"
else
  echo "room3d: wasm (dev) $before B"
fi
