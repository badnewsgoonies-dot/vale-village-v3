#!/usr/bin/env bash
set -euo pipefail

BINARY_NAME="vale-village"
OUT_DIR="./web"
PROFILE="wasm-release"

echo "=== Building for WASM ==="
cargo build --profile ${PROFILE} --target wasm32-unknown-unknown

echo "=== Running wasm-bindgen ==="
mkdir -p ${OUT_DIR}
wasm-bindgen --no-typescript --target web \
    --out-dir ${OUT_DIR}/ \
    --out-name "${BINARY_NAME}" \
    ./target/wasm32-unknown-unknown/${PROFILE}/${BINARY_NAME}.wasm

echo "=== Running wasm-opt ==="
if command -v wasm-opt &>/dev/null; then
    wasm-opt -Oz --enable-reference-types \
        -o ${OUT_DIR}/${BINARY_NAME}_bg.wasm \
        ${OUT_DIR}/${BINARY_NAME}_bg.wasm
    echo "wasm-opt applied"
else
    echo "wasm-opt not found, skipping (install binaryen for smaller output)"
fi

echo "=== Copying assets ==="
cp -r assets ${OUT_DIR}/ 2>/dev/null || true
cp -r data ${OUT_DIR}/ 2>/dev/null || true

echo "=== Build complete ==="
ls -lh ${OUT_DIR}/${BINARY_NAME}_bg.wasm
echo "Serve with: python3 -m http.server -d ${OUT_DIR}"
