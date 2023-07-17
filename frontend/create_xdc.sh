#!/bin/sh

set -e

DESTDIR="$PWD/../assets"

mkdir -p "$DESTDIR"
cd dist

echo "Building store.xdc"
zip -9 --recurse-paths "store.xdc" *
cp store.xdc "$DESTDIR"
