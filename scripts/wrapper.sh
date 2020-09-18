#!/usr/bin/env bash

WRAPPER_CMD=""
for line in $@; do WRAPPER_CMD="${WRAPPER_CMD} ${line}"; done
source ~/.profile
cd btrfsutil-rs
printf "%s\n" "----> Wrapper:${WRAPPER_CMD}"
exec bash -c "${WRAPPER_CMD}"
