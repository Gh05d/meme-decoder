#!/usr/bin/env bash
set -e

# 1. Build the WASM package into “pkg/”
wasm-pack build --target web --out-dir pkg --release

# 2. Remove the autogenerated .gitignore from pkg/
rm -f pkg/.gitignore

# 3. If the parent repo exists at ../meme-farmer, replace its meme-decoder folder
if [ -d "../meme-farmer" ]; then
  rm -rf ../meme-farmer/meme-decoder
  cp -r pkg ../meme-farmer/meme-decoder
fi
