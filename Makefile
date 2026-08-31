.PHONY: trunk build serve test b c t

build: trunk
	trunk build

serve: trunk
	trunk serve

trunk:
	trunk --version || cargo install trunk --locked

# test on host, not on wasm
test:
	cargo test -q --target=host-tuple

fmt:
	cargo fmt

check:
	cargo check --tests
	cargo clippy

b: build
c: check
t: test
