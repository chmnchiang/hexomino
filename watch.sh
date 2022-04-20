#!/bin/sh
trap 'echo kill -2 $p1; kill -2 $p1' SIGINT
trunk serve & p1=$!
cargo watch -x "run --bin hexomino-server --features internal-debug"
