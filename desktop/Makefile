.PHONY: check test build

check:
	npm run build
	npm run test
	cargo check --manifest-path src-tauri/Cargo.toml
	cargo test --manifest-path src-tauri/Cargo.toml

test:
	npm run test
	cargo test --manifest-path src-tauri/Cargo.toml

build:
	npm run build
