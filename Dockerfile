# syntax=docker/dockerfile:1.0.0-experimental
FROM rust:1.42 as cargo-build

ENV LLVM_CONFIG_PATH /usr/lib/llvm-7/bin/llvm-config

RUN mkdir /build
WORKDIR /build

RUN curl "https://awscli.amazonaws.com/awscli-exe-linux-x86_64.zip" -o "awscliv2.zip" && \
    unzip awscliv2.zip && \
    ./aws/install && \
    rm -rf aws awscliv2.zip

RUN apt-get update && \
    apt-get install -y \
        build-essential \
        clang \
        cmake \
        zip \
        libuv1-dev \
        binutils-dev \
        libcurl4-openssl-dev \
        libiberty-dev \
        libelf-dev \
        libdw-dev \
        jq \
        google-perftools

RUN for each in $(find /usr/lib/x86_64-linux-gnu -name libtcmalloc.so.\* | head -n1); do ln -s $each /usr/lib/x86_64-linux-gnu/libtcmalloc.so; done

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- --no-modify-path --default-toolchain none -y && \
    rustup component add clippy rustfmt

RUN git clone https://github.com/aws/aws-greengrass-core-sdk-c && \
    cd aws-greengrass-core-sdk-c && \
    mkdir build && \
    cd build && \
    cmake .. && \
    CC=arm-linux-gnueabihf-gcc make && \
    CC=arm-linux-gnueabihf-gcc make install

RUN rustup install nightly

RUN which cargo-make || cargo install --debug cargo-make
RUN which grcov || cargo +nightly install --debug grcov
RUN useradd rust --user-group --create-home --shell /bin/bash --groups sudo

WORKDIR /home/rust/src
