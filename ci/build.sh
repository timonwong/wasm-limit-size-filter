#!/bin/bash

set -eux

# 配置 Rustup Mirror (Bytedance)
export RUSTUP_DIST_SERVER="https://rsproxy.cn"
export RUSTUP_UPDATE_ROOT="https://rsproxy.cn/rustup"

# 配置 creates.io Mirror (Bytedance)
mkdir -p $HOME/.cargo
cat <<EOF | tee $HOME/.cargo/config
[source.crates-io]
replace-with = 'rsproxy'

[source.rsproxy]
registry = "https://rsproxy.cn/crates.io-index"
EOF

make build
