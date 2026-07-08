#!/bin/bash

set -euo pipefail

cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
