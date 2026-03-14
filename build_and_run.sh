#!/bin/bash

set -e

rm -f clouder

BUILD_DIR="target/release"
cargo build --all-features
cp "${BUILD_DIR}/clouder" .
./clouder
