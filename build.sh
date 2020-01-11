#!/bin/sh

NAME="aws_greengrass_core_sdk_rust"

usage() {
  cat << EOF >&2
Usage: $PROGNAME [-c|-nc|-h]

 -c : command (build, test)
 -nc : no cache
 -h : this message

EOF
  exit 1
}

command=""
dir=default_dir file=default_file verbose_level=0
while getopts "nc:c:" o; do
  case $o in
    (nc) nocache="--no-cache";;
    (c) command=${OPTARG};;
    # (d) dir=$OPTARG;;
    # (v) verbose_level=$((verbose_level + 1));;
    (*) usage
  esac
done
shift "$((OPTIND - 1))"

mkdir ./target > /dev/null 2>&1
cargo_version=$(grep -e '^version' Cargo.toml | awk '{print $3}' | sed -e 's/\"//g')
docker build $nocache --build-arg CARGO_VERSION:$cargo_version -t $NAME .
docker_exit_code=$?

if [ $docker_exit_code -eq 0 ]; then 
    case $command in 
        "test")
            docker run -it --entrypoint cargo $NAME -- "test"
            ;;
        "shell")
            docker run -it --entrypoint /bin/bash $NAME
            ;;
        *)
            echo "Copying to /target/docker_release"
            docker run --rm -v $(pwd):/data $NAME
            ;;
    esac
else 
    >&2 echo "$0 exited with status $docker_exit_code" 
fi