#!/bin/bash
set -e

# Change to project root directory
cd "$(dirname "$0")/.."

cargo build --release --bin ctl-resource-manager
sudo ./target/release/ctl-resource-manager