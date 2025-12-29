#!/bin/bash

set -e

BUILD_DIR="target/release"
cargo build --release --all-features
cp "$BUILD_DIR/clouder" .
./clouder
