#!/bin/sh

NAME=aws_greengrass_core_sdk_rust

docker build -t $NAME .

docker run --rm -v $(pwd):/data $NAME


