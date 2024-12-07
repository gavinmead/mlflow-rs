.PHONY: clean test fmt

clean:
	cargo clean

test: fmt
	cargo test

fmt:
	cargo fmt --all -- --check

all: test