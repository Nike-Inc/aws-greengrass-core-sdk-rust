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
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock
COPY ./build.rs ./build.rs
COPY ./wrapper.h ./wrapper.h
RUN mkdir src && \
    echo "fn main() {println!(\"if you see this, the build broke: ${CARGO_VERSION}\")}" > src/main.rs
RUN cargo build --release

# FROM rust:1.40
# COPY --from=cargo-build /build .
# WORKDIR /build
COPY ./scripts/copyzip.sh ./
COPY ./src ./src
RUN cargo build --release

VOLUME [ "/data" ]
ENTRYPOINT [ "./copyzip.sh"]