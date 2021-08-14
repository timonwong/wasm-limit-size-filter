.PHONY: build
build:
	rustup target add wasm32-unknown-unknown && \
        cargo build --target=wasm32-unknown-unknown --release && \
        cp target/wasm32-unknown-unknown/release/add_header_rs.wasm filter.wasm

.PHONY: e2e-test
e2e-test:
	go mod download
	go test ./test/...

.PHONY:
clean:
	rm -rf target
	rm -f filter.wasm
