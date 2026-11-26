#!/usr/bin/env bash
# Build vỏ web ra crates/web/dist. Chạy từ bất kỳ đâu.
#   ./build.sh          bản đo (profile wasm-release: opt-z, LTO, wasm-opt -Oz, brotli)
#   ./build.sh dev      bản lặp nhanh (không tối ưu, không wasm-opt)
# Phục vụ: python -m http.server -d crates/web/dist 8091
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
here=$(cd "$(dirname "$0")" && pwd)
root=$(cd "$here/../.." && pwd)
profile=${1:-wasm-release}
cargo build --manifest-path "$root/Cargo.toml" -p web --lib \
  --target wasm32-unknown-unknown --profile "$profile"
out="$root/target/wasm32-unknown-unknown/$([ "$profile" = dev ] && echo debug || echo "$profile")/web.wasm"
dist="$here/dist"
rm -rf "$dist" && mkdir "$dist" && cp "$here/index.html" "$dist/"
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
