# Checks two given strings for equality.
eq = $(if $(or $(1),$(2)),$(and $(findstring $(1),$(2)),\
                                $(findstring $(2),$(1))),1)

check:
	SKIP_WASM_BUILD=1 cargo check

test:
	SKIP_WASM_BUILD=1 cargo test --all

run.to_bsc:
	 cargo run --release realis-to-bsc

run.to_realis:
	 cargo run --release bsc-to-realis

build:
	 cargo build --release

# Format Rust sources with rustfmt.
#
# Usage:
#	make fmt [check=(no|yes)]

fmt:
	SKIP_WASM_BUILD=1 cargo +nightly fmt --all $(if $(call eq,$(check),yes),-- --check,)

lint:
	SKIP_WASM_BUILD=1 cargo clippy --workspace -- -D clippy::pedantic -D warnings

.PHONY: lint fmt build run.to_bsc run.to_realis test check
