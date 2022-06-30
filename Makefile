.PHONY: setup
setup:
	cargo install wasm-snip
	rustup component add clippy

.PHONY: build
build:
	rustup target add wasm32-unknown-unknown && \
        cargo build --target=wasm32-unknown-unknown --release && \
	 	wasm-snip -o filter.wasm target/wasm32-unknown-unknown/release/limit_size_rs.wasm

.PHONY: e2e-test
e2e-test:
	go mod download
	go test ./test/...

.PHONY:
clean:
	rm -rf target
	rm -f filter.wasm
