#!/bin/sh
# A script to copy the generated rust bindings to the target directory
NAME=aws_greengrass_core_sdk_rust
mkdir target > /dev/null 2>&1
docker run --rm -v $(pwd):/data --entrypoint '/bin/sh' $NAME -c "find ./target -name bindings.rs -exec cp {} /data/target/ \;"
rustfmt ./target/bindings.rs