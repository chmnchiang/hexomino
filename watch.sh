#!/bin/sh
trap 'echo kill $p1 $p2; kill $p1 $p2' SIGINT
trunk serve & p1=$!
cargo watch -x "run --bin hexomino-server" & p2=$!
wait $p1 $p2
