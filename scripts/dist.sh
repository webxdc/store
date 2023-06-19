#!/bin/sh
#
# Script to create the distribution tarball.
set -e

SRC="$PWD"

# Build the frontend.
cd "$SRC/frontend"
npx pnpm install

TMP="$(mktemp -d)"
export DESTDIR="$TMP/appstore-bot"
mkdir "$DESTDIR"

npm run build

# Build the backend.
cd "$SRC"
cargo build --release

cp target/release/github-bot "$DESTDIR/appstore-bot"
git describe --always >"$DESTDIR/bot-data/VERSION"

mkdir -p "$SRC/dist"
OUT="$SRC/dist/appstore-bot.tar.gz"
tar -C "$TMP" -czf "$OUT" appstore-bot

echo Distribution tarball is built at $OUT >&2

rm -fr "$TMP"
