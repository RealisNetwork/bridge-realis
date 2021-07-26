.PHONY: check
check:
	SKIP_WASM_BUILD=1 cargo check --release

.PHONY: test
test:
	SKIP_WASM_BUILD=1 cargo test --release --all

.PHONY: run to-bsc
run:
	 cargo run --release realis-to-bsc

.PHONY: run to-realis
run:
	 cargo run --release bsc-to-realis

.PHONY: build
build:
	 cargo build --release

.PHONY: fmt
fmt:
	cargo fmt -p realis-adapter -p realis-sender -p bsc-adapter -p bsc-sender

.PHONY: clippy
clippy:
	cargo clippy
