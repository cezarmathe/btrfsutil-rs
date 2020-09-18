#!/usr/bin/env bash

printf "%s\n" "----> Running btrfsutil_testenv_destroy"

if [[ -z "${BTRFSUTIL_TESTENV}" ]]; then
    printf "%s\n%s\n%s\n%s\n" '***WARNING***' \
        'Testing `btrfsutil` can result in permanent system damage.' \
        'Only test `btrfsutil` in disposable environments.' \
        'TESTING IS NOT ALLOWED'
    exit 1
fi

BTRFSUTIL_TESTENV_EXIT_CODE=0
sudo umount "/mnt/${BTRFSUTIL_TESTENV}"
if [[ "$?" != "0" ]]; then BTRFSUTIL_TESTENV_EXIT_CODE=1; fi
sudo rm -r "/mnt/${BTRFSUTIL_TESTENV}"
if [[ "$?" != "0" ]]; then BTRFSUTIL_TESTENV_EXIT_CODE=1; fi

rm "${HOME}/${BTRFSUTIL_TESTENV}.img"
if [[ "$?" != "0" ]]; then BTRFSUTIL_TESTENV_EXIT_CODE=1; fi

exit ${EXIT_CODE}
