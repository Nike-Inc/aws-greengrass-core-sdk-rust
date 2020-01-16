FROM rust:1.40 as initial-build

RUN mkdir /build
WORKDIR /build

RUN apt-get update && \
    apt-get install -y \
        build-essential \        
        clang \
        cmake \
        libuv1-dev && \
    git clone https://github.com/aws/aws-greengrass-core-sdk-c.git && \
    cd aws-greengrass-core-sdk-c && \
    mkdir build && \
    cd build && \
    cmake .. && \
    make && \
    make install 

ARG CARGO_VERSION

ENV LLVM_CONFIG_PATH "/usr/lib/llvm-7/bin/llvm-config"

WORKDIR /build
ADD ./Cargo.toml ./Cargo.toml
ADD ./Cargo.lock ./Cargo.lock
ADD ./build.rs ./build.rs
ADD ./wrapper.h ./wrapper.h
RUN mkdir src && \
    echo "fn foo() {println!(\"if you see this, the build broke: ${CARGO_VERSION}\")}" > src/lib.rs
RUN cargo build --release

ADD ./scripts/copytarget.sh ./
ADD ./src ./src
RUN touch src/lib.rs && \
    cargo build --release

VOLUME [ "/data" ]
ENTRYPOINT [ "./copytarget.sh"]