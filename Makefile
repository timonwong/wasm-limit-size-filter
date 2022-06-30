MKFILE_PATH := $(abspath $(lastword $(MAKEFILE_LIST)))
PROJECT_PATH := $(patsubst %/,%,$(dir $(MKFILE_PATH)))

ifeq ($(OS),Windows_NT)
	PLATFORM = windows

	ifeq ($(PROCESSOR_ARCHITEW6432),AMD64)
		HOST_ARCH = x64
	else
		ifeq ($(PROCESSOR_ARCHITECTURE),AMD64)
			HOST_ARCH = x86_64
		endif
		ifeq ($(PROCESSOR_ARCHITECTURE),x86)
			HOST_ARCH = x86
		endif
	endif
else
	ifeq ($(shell uname -s),Linux)
		PLATFORM = linux

		ifeq ($(shell uname -m),x86_64)
			HOST_ARCH = x86_64
		endif
		ifneq ($(filter %86,$(shell uname -m)),)
			HOST_ARCH = x86
		endif
		ifeq ($(shell uname -m),armv7l)
			HOST_ARCH = armv7hf
		endif
		ifeq ($(shell uname -m),aarch64)
			HOST_ARCH = aarch64
		endif
		ifeq ($(shell uname -m),armv8)
			HOST_ARCH = aarch64
		endif
		ifeq ($(shell uname -m),arm64)
			HOST_ARCH = aarch64
		endif
	endif
	ifeq ($(shell uname -s),Darwin)
		PLATFORM = macos

		ifeq ($(shell uname -m),x86_64)
			HOST_ARCH = x86_64
		endif
		ifeq ($(shell uname -m),arm64)
			HOST_ARCH = aarch64
		endif
	endif
endif

ifndef PLATFORM
$(error We could not detect your host platform)
endif
ifndef HOST_ARCH
$(error We could not detect your host architecture)
endif

BINARYEN_TAG=version_109

.PHONY: setup
setup:
	@echo "Install wasm target"
	rustup target add wasm32-unknown-unknown
	@echo "Install wasm-snip tool"
	cargo install wasm-snip
	@echo "Install binaryen tool"
	@mkdir -p .tools/binaryen
	curl -Lo .tools/binaryen.tar.gz \
		"https://github.com/WebAssembly/binaryen/releases/download/$(BINARYEN_TAG)/binaryen-$(BINARYEN_TAG)-$(HOST_ARCH)-$(PLATFORM).tar.gz" \
		&& tar -xzf .tools/binaryen.tar.gz -C .tools/binaryen --strip-component=1 && rm .tools/binaryen.tar.gz
	@#echo "Install clippy linter"
	@#rustup component add clippy


.PHONY: release
release: export BUILD?=release
release:  ## Build release WASM filter
	$(MAKE) build

.PHONY: build
build: export TARGET?=wasm32-unknown-unknown
build: export BUILD?=debug
build:  ## Build WASM filter
	cargo build --target=$(TARGET) --release $(CARGO_EXTRA_ARGS)
	if test "x$(BUILD)" = "xrelease"; then \
  		wasm-snip -o $(PROJECT_PATH)/target/$(TARGET)/$(BUILD)/snipped.wasm $(PROJECT_PATH)/target/$(TARGET)/$(BUILD)/wasm_limit_size_filter.wasm && \
  		.tools/binaryen/bin/wasm-opt -O4 --dce -o $(PROJECT_PATH)/plugin.wasm $(PROJECT_PATH)/target/$(TARGET)/$(BUILD)/snipped.wasm; \
  	fi

.PHONY: e2e-test
e2e-test:
	go mod download
	go test ./test/...

.PHONY:
clean:
	cargo clean
	rm -f filter.wasm
