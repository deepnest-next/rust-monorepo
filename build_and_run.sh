#!/bin/bash
set -e

echo "Building common API..."

pushd common
cargo build
popd

echo "Building plugin_svg..."

pushd plugin_svg
cargo build --target-dir ../target_svg
popd

echo "Building plugin_math..."

pushd plugin_math
cargo build --target-dir ../target_math
popd

echo "Building main..."

pushd main
cargo run
popd