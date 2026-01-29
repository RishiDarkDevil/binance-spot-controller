echo 1024 | sudo tee /sys/kernel/mm/hugepages/hugepages-2048kB/nr_hugepages
cargo build --release --bin ctl-resource-manager
sudo ../target/release/ctl-resource-manager