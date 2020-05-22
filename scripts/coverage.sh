#!/bin/sh
# Builds coverage with the grcov tool

# grcov currently only works with the nightly toolchain
TOOLCHAIN="nightly"
CARGO="cargo $TOOLCHAIN"
COVERAGE_DIRECTORY="./target/debug/coverage/"

chain_result=$(rustup toolchain list | grep $TOOLCHAIN)

if [ -z "${chain_result+x}" ]; then
    echo "toolchain ${TOOLCHAIN} not installed, installing"
    rustup toolchain install $TOOLCHAIN
fi  

which grcov > /dev/null 2>&1

if [ $? -eq 1 ]; then
    echo "grcov not installed.. installing"
    $CARGO install grcov
fi

export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort"
export RUSTDOCFLAGS="-Cpanic=abort"

$CARGO build --features coverage
$CARGO test --features coverage

grcov ./target/debug/ -s . -t html --llvm --branch --ignore-not-existing -o $COVERAGE_DIRECTORY
