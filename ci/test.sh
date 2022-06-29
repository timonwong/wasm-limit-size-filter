#!/bin/bash

set -eux

# 安装 bazelisk (只是为了让 istio.io/proxy/test 工作)
export USE_BAZEL_VERSION="2.2.0"
export BAZELISK_BASE_URL="https://nexus-b.alauda.cn/repository/asm/bazel"

GO111MODULE=off go get github.com/bazelbuild/bazelisk && \
  cp "$(which bazelisk)" /usr/local/bin/bazel

# 使用 ISTIO_TEST_VERSION 环境变量可以让测试自动下载 envoy
ISTIO_TEST_VERSION=1.12.5 make e2e-test
