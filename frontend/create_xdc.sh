#!/bin/sh
mkdir -p ../bot-data
cd dist

echo "Building appstore.xdc"
cd shop
mv shop.html index.html
cp ../../build-files/shop/* .
zip -9 --recurse-paths "appstore.xdc" *
cp appstore.xdc ../../../bot-data
cd ..

echo "Building submit-helper.xdc"
cd submit
mv submit.html index.html
cp ../../build-files/submit/* .
zip -9 --recurse-paths "submit-helper.xdc" * 
cp submit-helper.xdc ../../../bot-data
cd ..

echo "Building review-helper.xdc"
cd review
mv review.html index.html
cp ../../build-files/review/* .
zip -9 --recurse-paths "review-helper.xdc" * 
cp review-helper.xdc ../../../bot-data
cd ..
