#!/bin/bash

set -e

rm -f clouder

BUILD_DIR="target/debug"
cargo build
cp "${BUILD_DIR}/clouder" .
./clouder
