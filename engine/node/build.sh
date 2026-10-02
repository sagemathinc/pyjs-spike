#!/bin/sh
# Builds the native addon into this directory: require("./engine/node").
set -e
cd "$(dirname "$0")"
cargo build --release -p sagebrush-node
cp ../target/release/libsagebrush_node.so sagebrush.node
echo "built $(pwd)/sagebrush.node"
