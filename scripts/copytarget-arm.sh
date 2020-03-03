#!/bin/sh
BINARY_NAME=$1
BINARY_PATH="/build/target/release/${BINARY_NAME}"
TARGET_PATH="/data/target/docker_release_arm/"

if [ ! -e "${TARGET_PATH}" ]
then
    2>&1 echo "Binary ${BINARY_PATH} does not exist"
    exit 1
fi

rm -rf "${TARGET_PATH}" > /dev/null 2>&1
mkdir -p "${TARGET_PATH}" > /dev/null 2>&1
echo "Copying ${BINARY_PATH} ${TARGET_PATH}"
cp "${BINARY_PATH}" "${TARGET_PATH}"