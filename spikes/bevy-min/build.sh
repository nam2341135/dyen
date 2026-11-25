#!/usr/bin/env bash
# Build wasm release + đo kích thước (ADR 0008 điều kiện 1). Chạy từ spikes/bevy-min.
set -euo pipefail
export PATH="$HOME/.cargo/bin:$PATH"
name=spike-bevy-min
cargo build --release --target wasm32-unknown-unknown
rm -rf dist && mkdir dist && cp web/index.html dist/
wasm-bindgen --target web --no-typescript --out-dir dist \
  target/wasm32-unknown-unknown/release/$name.wasm
raw=dist/${name}_bg.wasm
wasm-opt -Oz --enable-bulk-memory --enable-nontrapping-float-to-int --enable-sign-ext \
  --enable-mutable-globals --enable-reference-types -o dist/opt.wasm "$raw"
size() { stat -c %s "$1"; }
echo "wasm sau bindgen: $(size "$raw") B"
mv dist/opt.wasm "$raw"
echo "wasm sau wasm-opt -Oz: $(size "$raw") B"
brotli -q 11 -k -f "$raw"
echo "wasm brotli q11: $(size "$raw.br") B"
brotli -q 11 -k -f dist/$name.js
echo "js brotli q11: $(size dist/$name.js.br) B"
