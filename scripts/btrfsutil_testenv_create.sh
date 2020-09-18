#!/usr/bin/env bash

printf "%s\n" "----> Running btrfsutil_testenv_create"

if [[ -z "${BTRFSUTIL_TESTENV}" ]]; then
    printf "%s\n%s\n%s\n%s\n" '***WARNING***' \
        'Testing `btrfsutil` can result in permanent system damage.' \
        'Only test `btrfsutil` in disposable environments.' \
        'TESTING IS NOT ALLOWED'
    exit 1
fi

BTRFSUTIL_TESTENV_EXIT_CODE=0
fallocate -l 1G "${HOME}/${BTRFSUTIL_TESTENV}.img"
if [[ "$?" != "0" ]]; then BTRFSUTIL_TESTENV_EXIT_CODE=1; fi
mkfs -t btrfs -q "${HOME}/${BTRFSUTIL_TESTENV}.img"
if [[ "$?" != "0" ]]; then BTRFSUTIL_TESTENV_EXIT_CODE=1; fi

sudo mkdir "/mnt/${BTRFSUTIL_TESTENV}"
if [[ "$?" != "0" ]]; then BTRFSUTIL_TESTENV_EXIT_CODE=1; fi
sudo mount -t btrfs -o loop \
    "${HOME}/${BTRFSUTIL_TESTENV}.img" \
    "/mnt/${BTRFSUTIL_TESTENV}"
if [[ "$?" != "0" ]]; then BTRFSUTIL_TESTENV_EXIT_CODE=1; fi

sudo btrfs subvolume show "/mnt/${BTRFSUTIL_TESTENV}"
if [[ "$?" != "0" ]]; then BTRFSUTIL_TESTENV_EXIT_CODE=1; fi

exit ${BTRFSUTIL_TESTENV_EXIT_CODE}
