.PHONY: check test run clean

# check runs cargo test, which includes tests/public_examples_validation_guard.rs (v4.7.12+)
check:
	cargo fmt --check
	cargo clippy -- -D warnings
	cargo test
	cargo build

test:
	cargo test

run:
	cargo run

clean:
	cargo clean
