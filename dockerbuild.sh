#!/bin/sh
docker build -t aws-greengrass-core-sdk-rust-builder .
docker run --rm -it \
  -v "$(pwd)":/home/rust/src \
  -v "$(pwd)/.dockercache/cargo-git":/home/rust/.cargo/git \
  -v "$(pwd)/.dockercache/cargo-registry":/home/rust/.cargo/registry \
  -v "$(pwd)/target/docker":/home/rust/src/target \
  aws-greengrass-core-sdk-rust-builder $@