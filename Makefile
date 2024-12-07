.PHONY: clean test

clean:
	cargo clean

test:
	cargo test

all: test