.PHONY: clean test fmt

clean:
	cargo clean

test:
	cargo test

fmt:
	cargo fmt

all: test