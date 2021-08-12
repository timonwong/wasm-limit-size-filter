#!/bin/bash

set -eux

# 配置 Rustup Mirror
export RUSTUP_DIST_SERVER="https://mirrors.ustc.edu.cn/rust-static"
export RUSTUP_UPDATE_ROOT="https://mirrors.ustc.edu.cn/rust-static/rustup"

# 配置 creates.io Mirror
mkdir -p ~/.cargo
cat <<EOF | tee ~/.cargo/config
[source.crates-io]
replace-with = 'ustc'

[source.ustc]
registry = "https://mirrors.ustc.edu.cn/crates.io-index"
EOF

make build
