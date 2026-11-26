#!/usr/bin/env bash
# Build wasm + đo kích thước. Chạy từ spikes/lang-map.
#   ./build.sh          bản đo (release: opt-z, LTO, wasm-opt -Oz, brotli)
#   ./build.sh fast     bản lặp nhanh (opt 2, không LTO, không wasm-opt)
# Phục vụ: python -m http.server -d dist 8090
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
profile=${1:-release}
cargo build --profile "$profile" --target wasm32-unknown-unknown --lib
rm -rf dist && mkdir dist && cp web/index.html dist/
wasm-bindgen --target web --no-typescript --out-dir dist \
  "target/wasm32-unknown-unknown/$profile/lang_map.wasm"
size() { stat -c %s "$1"; }
raw=dist/lang_map_bg.wasm
before=$(size "$raw")
if [ "$profile" = release ]; then
  wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext \
    --enable-mutable-globals --enable-reference-types -o dist/opt.wasm "$raw"
  mv dist/opt.wasm "$raw"
  brotli -q 11 -k -f "$raw" dist/lang_map.js
  echo "wasm $before B → -Oz $(size "$raw") B → brotli $(size "$raw.br") B; js brotli $(size dist/lang_map.js.br) B"
else
  echo "wasm ($profile) $before B"
fi
