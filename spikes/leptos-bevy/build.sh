#!/usr/bin/env bash
# Build hai wasm (vỏ Leptos, 3D Bevy) + đo kích thước. Chạy từ spikes/leptos-bevy.
# Phục vụ: python -m http.server -d dist 8080
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
cargo build --release --target wasm32-unknown-unknown
rm -rf dist && mkdir dist && cp web/index.html dist/
size() { stat -c %s "$1"; }
for name in shell view3d; do
  wasm-bindgen --target web --no-typescript --out-dir dist \
    target/wasm32-unknown-unknown/release/$name.wasm
  raw=dist/${name}_bg.wasm
  before=$(size "$raw")
  wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext \
    --enable-mutable-globals --enable-reference-types -o dist/opt.wasm "$raw"
  mv dist/opt.wasm "$raw"
  brotli -q 11 -k -f "$raw" dist/$name.js
  echo "$name: wasm $before B → -Oz $(size "$raw") B → brotli $(size "$raw.br") B; js brotli $(size dist/$name.js.br) B"
done
