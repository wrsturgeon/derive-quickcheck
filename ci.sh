#!/bin/sh

set -eux

# export VERBOSE='--verbose'
export VERBOSE=

cargo fmt -- --check
cargo build --no-default-features ${VERBOSE}
cargo build --all-features ${VERBOSE}
set +u
if [ -z "${GITHUB_ACTIONS}" ]
then
  set -u
  export QUICKCHECK_TESTS=10000
  if cargo test ${VERBOSE} --no-default-features && cargo test ${VERBOSE} --all-features
  then
    : # all good
  else
    cat proptest-regressions/base.txt
    exit 1
  fi
else
  set -u
  export QUICKCHECK_TESTS=1000000
  if cargo test ${VERBOSE} --no-default-features && cargo test ${VERBOSE} --all-features
  then
    : # all good
  else
    cat proptest-regressions/base.txt
    exit 1
  fi
fi
cargo clippy --all-targets ${VERBOSE} --no-default-features
cargo clippy --all-targets ${VERBOSE} --all-features

cd qcderive-test || exit 0
. ../ci.sh
