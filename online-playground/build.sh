#!/bin/sh

wasm-pack build --target=web
mkdir -p dist/
cp index.html index.js pkg/*.js pkg/*.wasm dist/