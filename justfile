
#[default]
before-commit: fmt update check
check: fmt-check test clippy

build:
	cargo build --release

fmt:
	cargo fmt --

fmt-check:
	cargo fmt -- --check

test:
	cargo test

clippy:
	cargo clippy -- -D warnings -D clippy::pedantic --verbose --no-deps

clean:
	cargo clean

update:
	cargo update
