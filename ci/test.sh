#!/bin/bash

set -eux

# Install bazelisk just in order to let istio.io/proxy/test work
export USE_BAZEL_VERSION="2.2.0"
go install github.com/bazelbuild/bazelisk && cp "$(which bazelisk)" /usr/local/bin/bazel

# Download and test against istio releases automatically
ISTIO_TEST_VERSION=1.12.5 make e2e-test
