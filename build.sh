#!/bin/sh

NAME=aws_greengrass_core_sdk_rust

usage() {
  cat << EOF >&2
Usage: $PROGNAME [-c]

 -c : clear docker cache

EOF
  exit 1
}


dir=default_dir file=default_file verbose_level=0
while getopts c o; do
  case $o in
    (c) nocache="--no-cache";;
    # (d) dir=$OPTARG;;
    # (v) verbose_level=$((verbose_level + 1));;
    (*) usage
  esac
done
shift "$((OPTIND - 1))"

mkdir ./target > /dev/null 2>&1

docker build $nocache $1 -t $NAME .

docker run --rm -v $(pwd):/data $NAME


