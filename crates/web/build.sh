#!/usr/bin/env bash
# Build vỏ web ra crates/web/dist, rồi phòng 3D ra dist/room/. Chạy từ bất kỳ đâu.
#   ./build.sh                bản đo (profile wasm-release: opt-z, LTO, wasm-opt -Oz, brotli)
#   ./build.sh dev            bản lặp nhanh (không tối ưu, không wasm-opt)
#   ./build.sh [dev] --no-room  không build phòng 3D; giữ dist/room/ cũ nếu có
# Đối số theo thứ tự bất kỳ. Phục vụ: python -m http.server -d crates/web/dist 8091
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)
profile=wasm-release
room=1
for arg in "$@"; do
  case "$arg" in
    dev) profile=dev ;;
    --no-room) room=0 ;;
    *) echo "đối số lạ: $arg (dùng: [dev] [--no-room])" >&2; exit 2 ;;
  esac
done
cargo build --manifest-path "$root/Cargo.toml" -p web --lib \
  --target wasm32-unknown-unknown --profile "$profile"
out="$root/target/wasm32-unknown-unknown/$([ "$profile" = dev ] && echo debug || echo "$profile")/web.wasm"
dist="$here/dist"
keep=""
if [ "$room" = 0 ] && [ -d "$dist/room" ]; then
  keep=$(mktemp -d)
  cp -r "$dist/room" "$keep/"
fi
rm -rf "$dist" && mkdir "$dist" && cp "$here/index.html" "$dist/"
if [ -n "$keep" ]; then
  mv "$keep/room" "$dist/room" && rm -rf "$keep"
fi
wasm-bindgen --target web --no-typescript --out-dir "$dist" "$out"
size() { stat -c %s "$1"; }
raw="$dist/web_bg.wasm"
before=$(size "$raw")
if [ "$profile" != dev ]; then
  wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext \
    --enable-mutable-globals --enable-reference-types -o "$dist/opt.wasm" "$raw"
  mv "$dist/opt.wasm" "$raw"
  brotli -q 11 -k -f "$raw" "$dist/web.js"
  echo "wasm $before B → -Oz $(size "$raw") B → brotli $(size "$raw.br") B; js brotli $(size "$dist/web.js.br") B"
else
  echo "wasm (dev) $before B"
fi
if [ "$room" = 1 ]; then
  if [ "$profile" = dev ]; then bash "$root/crates/room3d/build.sh" dev
  else bash "$root/crates/room3d/build.sh"; fi
fi
