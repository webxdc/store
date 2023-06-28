#!/bin/sh

: "${DESTDIR:=$PWD/..}"

DESTDIR="$DESTDIR/assets"

mkdir -p "$DESTDIR"
cd dist

echo "Building store.xdc"
cd shop
mv shop.html index.html
cp ../../build-files/shop/* .
zip -9 --recurse-paths "store.xdc" *
cp store.xdc "$DESTDIR"
cd ..

echo "Building submit-helper.xdc"
cd submit
mv submit.html index.html
cp ../../build-files/submit/* .
zip -9 --recurse-paths "submit-helper.xdc" * 
cp submit-helper.xdc "$DESTDIR"
cd ..

echo "Building review-helper.xdc"
cd review
mv review.html index.html
cp ../../build-files/review/* .
zip -9 --recurse-paths "review-helper.xdc" * 
cp review-helper.xdc "$DESTDIR"
cd ..
