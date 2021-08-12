.PHONY: build
build:
	rustup target add wasm32-unknown-unknown && \
        cargo build --target=wasm32-unknown-unknown --release && \
        cp target/wasm32-unknown-unknown/release/add_header_rs.wasm extension.wasm

.PHONY: e2e-test
e2e-test:
	go mod download
	go test -v ./tests/...

.PHONY:
clean:
	rm -rf target
	rm -f extension.wasm
