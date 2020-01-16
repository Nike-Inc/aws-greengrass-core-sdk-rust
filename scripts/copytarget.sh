#!/bin/sh
rm -rf /data/target/docker_release
mkdir /data/target > /dev/null 2>&1
cp -r /build/target/release /data/target/docker_release 