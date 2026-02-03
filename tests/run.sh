#!/bin/bash
set -e
cd "$(dirname "$0")"

# Build the project
echo "Building wrun..."
cargo build --release

WRUN="../target/release/wrun"

echo "------------------------------------------------"
echo "Running noop.wat tests (wasmtime to pocket-ic)"
echo "------------------------------------------------"

echo "[noop] Running with wasmtime..."
$WRUN data/noop.wat noop --instance-type wasmtime

echo "[noop] Running with embedder..."
$WRUN data/noop.wat noop --instance-type embedder

echo "[noop] Running with execenv..."
$WRUN data/noop.wat noop --instance-type execenv

echo "[noop] Running with pocket-ic..."
$WRUN data/noop.wat noop --instance-type pocket-ic

echo ""
echo "------------------------------------------------"
echo "Running counter.wat tests (embedder to pocket-ic)"
echo "------------------------------------------------"

echo "[counter] Running with embedder..."
$WRUN data/counter.wat inc --instance-type embedder

echo "[counter] Running with execenv..."
$WRUN data/counter.wat inc --instance-type execenv

echo "[counter] Running with pocket-ic..."
$WRUN data/counter.wat inc --instance-type pocket-ic

echo ""
echo "All tests completed successfully."
