#!/bin/bash

VERSION=$1

if [ -z "$VERSION" ]; then
  echo "Usage: $0 <version>"
  exit 1
fi

mkdir -p bin/$VERSION

targets=("x86_64-apple-darwin" "aarch64-apple-darwin" "x86_64-unknown-linux-gnu")

for target in "${targets[@]}"; do
  echo "Building for target: $target"
  
  CARGO_TARGET_DIR="./bin/$VERSION" LEV_VERSION=$VERSION cross build --release --target $target -p lev
done

echo "Done"
