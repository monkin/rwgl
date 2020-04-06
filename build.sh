#!/bin/sh

set -ex

wasm-pack build --target web
python -m http.server