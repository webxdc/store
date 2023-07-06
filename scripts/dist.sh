#!/bin/sh
#
# Script to create the distribution tarball.

set -e

SRC="$PWD"

# Build the frontend.
cd "$SRC/frontend"
pnpm install
pnpm build

# Build the backend.
cd "$SRC"
cargo build --target x86_64-unknown-linux-musl --release

TMP="$(mktemp -d)"
DESTDIR="$TMP/xdcstore"
mkdir "$DESTDIR"
cp target/x86_64-unknown-linux-musl/release/xdcstore "$DESTDIR/xdcstore"

mkdir -p "$SRC/dist"
OUT="$SRC/dist/xdcstore.tar.gz"
tar -C "$TMP" -czf "$OUT" xdcstore 

echo Distribution tarball is built at $OUT >&2

rm -fr "$TMP"
