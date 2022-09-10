# Build and run the game
run:
    cargo run

# Build the game and serve it in the browser using WASM
serve:
    cargo run --target wasm32-unknown-unknown

# Build and run the game, restarting it whenever changes are detected
watch:
    cargo watch -x run