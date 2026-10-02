#!/bin/sh
# Builds the native addon into this directory: require("./engine/node").
set -e
cd "$(dirname "$0")"
cargo build --release -p modsym-node
cp ../target/release/libmodsym_engine_node.so modsym_engine.node
echo "built $(pwd)/modsym_engine.node"
