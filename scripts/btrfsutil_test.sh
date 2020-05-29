#!/usr/bin/env bash

source ~/.profile

${HOME}/btrfsutil_testenv_bootstrap.sh
if [[ "$?" != "0" ]]; then
    ${HOME}/btrfsutil_testenv_teardown.sh
    exit 1
fi

TESTS_ALL_SUCCESSFUL=0
cd btrfsutil-rs
cargo test
if [[ "$?" != "0" ]]; then
    TESTS_ALL_SUCCESSFUL=1
fi

${HOME}/btrfsutil_testenv_teardown.sh
if [[ "$?" != "0" ]]; then exit 1; fi
if [[ "${TESTS_ALL_SUCCESSFUL}" != "0" ]]; then exit 1; fi
