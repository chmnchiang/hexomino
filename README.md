## Release Build

Run
```
trunk build --release
cargo build --release --bin hexomino-server
```
Then, one can run
```
cargo run --release --bin hexomino-server
```
to start the server. An http server (e.g., nginx) should serve the static files
under `dist/`.
