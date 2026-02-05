#!/bin/bash
set -e

# Change to project root directory
cd "$(dirname "$0")/.."

cargo build --release --bin ctl-md-subscriber
sudo ./target/release/ctl-md-subscriber