run:
	cargo run

serve:
	cargo run --target wasm32-unknown-unknown

watch:
	cargo watch -x run

.PHONY: run serve watch