FROM artifactory.nike.com:9002/eap/rust-greengrass-base:v1

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
