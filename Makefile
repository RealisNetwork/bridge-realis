# Checks two given strings for equality.
eq = $(if $(or $(1),$(2)),$(and $(findstring $(1),$(2)),\
                                $(findstring $(2),$(1))),1)

check:
	SKIP_WASM_BUILD=1 cargo check

test:
	SKIP_WASM_BUILD=1 cargo test --all -- --nocapture --test-threads 1

run:
	SKIP_WASM_BUILD=1 cargo run --release

build:
	SKIP_WASM_BUILD=1 cargo build --release

udeps:
	SKIP_WASM_BUILD=1 cargo +nightly udeps

docker.build:
	docker build . -t bridge-realis
# Format Rust sources with rustfmt.
#
# Usage:
#	make fmt [check=(no|yes)]

fmt:
	SKIP_WASM_BUILD=1 cargo +nightly fmt --all $(if $(call eq,$(check),yes),-- --check,)

lint:
	SKIP_WASM_BUILD=1 cargo clippy --workspace -- -D clippy::pedantic -D warnings

.PHONY: lint fmt build run test check
