#!/bin/sh

BASE_DOCKER_VERSION=v1
WORKING_DIRECTORY='/src'

if [ "$#" -eq 1 ]; then
  EXAMPLE="$1"
  CARGO_LINE="--example ${EXAMPLE}"
else
  CARGO_LINE="--release"
fi

CACHE_DIRECTORY=~/tmp/aws-greengrass-core-sdk-rust

mkdir -p ~/tmp/ > /dev/null 2>&1

USER=$(id -u)
GROUP=$(id -g)

echo $CARGO_LINE

docker run --rm --user "root:root" \
  -e CARGO_HOME:/cargo \
  -v "$PWD":"${WORKING_DIRECTORY}" \
  -v $CACHE_DIRECTORY/cargo-git:/usr/local/cargo/git \
  -v $CACHE_DIRECTORY/cargo-registry:/usr/local/cargo/registry \
  -v $CACHE_DIRECTORY/target:"${WORKING_DIRECTORY}/target" \
  -w $WORKING_DIRECTORY artifactory.nike.com:9002/eap/rust-greengrass-base:$BASE_DOCKER_VERSION cargo build $CARGO_LINE

if [ "${EXAMPLE+1}" ]; then
    zip -j ./target/$EXAMPLE.zip ~/tmp/aws-greengrass-core-sdk-rust/target/debug/examples/$EXAMPLE
fi