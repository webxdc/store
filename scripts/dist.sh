#!/bin/sh
#
# Script to create the distribution tarball.
set -e

SRC="$PWD"

# Build the frontend.
cd "$SRC/frontend"
npx pnpm install

TMP="$(mktemp -d)"
export DESTDIR="$TMP/xdcstore"
mkdir "$DESTDIR"

npm run build

# Build the backend.
cd "$SRC"
cargo build --release

cp target/release/xdcstore "$DESTDIR/xdcstore"

mkdir -p "$SRC/dist"
OUT="$SRC/dist/xdcstore.tar.gz"
tar -C "$TMP" -czf "$OUT" xdcstore 

echo Distribution tarball is built at $OUT >&2

rm -fr "$TMP"
