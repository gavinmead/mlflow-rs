.PHONY: clean test fmt clippy

clean:
	cargo clean

test: fmt
	cargo test

fmt:
	cargo fmt --all

clippy:
	cargo clippy --workspace --tests --examples -- -D warnings

all: test