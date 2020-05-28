#!/usr/bin/env bash

source ~/.profile

/home/vagrant/btrfsutil_testenv_bootstrap.sh
if [[ "$?" != "0" ]]; then
    /home/vagrant/btrfsutil_testenv_teardown.sh
    exit 1
fi

printf "\n"
TARGET="$1"; shift
if [[ -z "${TARGET}" ]]; then
    printf "%s\n" "----> Missing TARGET parameter."
    exit 1
fi
printf "%s\n" "----> Testing target ${TARGET}"

TESTS=""
TESTS_ALL_SUCCESSFUL=0
function find_tests() {
    for test_src in $(ls /home/vagrant/btrfsutil-rs/tests/); do
        local test_name="$(basename ${test_src} .rs)"
        local bin_name="$(ls btrfsutil-rs/target/${TARGET}/ | grep ${test_name} | head -1)"
        if [[ -z "${bin_name}" ]]; then
            printf "%s\n" "----> No test found for test source file: ${test_src}"
            TESTS_ALL_SUCCESSFUL=1
        else
            TESTS="${TESTS} ${bin_name}"
        fi
    done
}
find_tests
printf "%s:%s\n" "----> Test binaries found" "${TESTS}"
for test in ${TESTS[@]}; do
    printf "%s: %s\n" "----> Running tests from binary" "${test}"
    ./btrfsutil-rs/target/"${TARGET}"/"${test}"
    if [[ "$?" != "0" ]]; then TESTS_ALL_SUCCESSFUL=1; fi
done

/home/vagrant/btrfsutil_testenv_teardown.sh
if [[ "$?" != "0" ]]; then exit 1; fi
if [[ "${TESTS_ALL_SUCCESSFUL}" != "0" ]]; then exit 1; fi
