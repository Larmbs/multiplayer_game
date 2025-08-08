#!/bin/bash

set -e

# Paths
OUTPUT_DIR="./build"

CLIENT_DIR="client"
SERVER_DIR="server"
LAUNCHER_DIR="launcher"

CLIENT_BUILD="target/release/client"
SERVER_BUILD="target/release/server"
LAUNCHER_BUILD="target/release/launcher"

# Clean previous output
rm -rf "$OUTPUT_DIR"
mkdir -p "$OUTPUT_DIR"

echo "Building client..."
cargo build --release --bin "$CLIENT_DIR"

echo "Building server..."
cargo build --release --bin "$SERVER_DIR"

echo "Building launcher..."
cargo build --release --bin "$LAUNCHER_DIR"

# Copy files and create zip for client
mkdir -p "$OUTPUT_DIR/client/"
cp "$CLIENT_BUILD" "$OUTPUT_DIR/client/"
cp "$CLIENT_DIR/version.txt" "$OUTPUT_DIR/client/"
cd "$OUTPUT_DIR/client"
zip -r ./client.zip ./client*
cd - > /dev/null

# Copy files and create zip for server
mkdir -p "$OUTPUT_DIR/server/"
cp "$SERVER_BUILD" "$OUTPUT_DIR/server/"
cp "$SERVER_DIR/version.txt" "$OUTPUT_DIR/server/"
cd "$OUTPUT_DIR/server"
zip -r ./server.zip ./server*
cd - > /dev/null

mkdir -p "$OUTPUT_DIR/launcher/"
cp "$LAUNCHER_BUILD" "$OUTPUT_DIR/launcher/"
cp "$LAUNCHER_DIR/version.txt" "$OUTPUT_DIR/launcher/"
cd "$OUTPUT_DIR/launcher"
zip -r ./launcher.zip ./launcher*
cd - > /dev/null

echo "✅ Build and packaging complete. Output in $OUTPUT_DIR"
