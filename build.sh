#!/bin/sh

NAME=aws_greengrass_core_sdk_rust

usage() {
  cat << EOF >&2
Usage: $PROGNAME [-nocache]

 -nocache : ...

EOF
  exit 1
}


dir=default_dir file=default_file verbose_level=0
while getopts nocache o; do
  case $o in
    (nocache) nocache="--no-cache";;
    # (d) dir=$OPTARG;;
    # (v) verbose_level=$((verbose_level + 1));;
    (*) usage
  esac
done
shift "$((OPTIND - 1))"

docker build $1 -t $NAME .

docker run --rm -v $(pwd):/data $NAME


