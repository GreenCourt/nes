.PHONY: trunk build serve test b c t

build: trunk
	trunk build

serve: trunk
	trunk serve

trunk:
	trunk --version || cargo install trunk --locked

# test on host, not on wasm
test:
	cargo test -q

fmt:
	cargo fmt

check:
	cargo check --target=wasm32-unknown-unknown
	cargo check --tests
	cargo clippy

b: build
c: check
t: test
