#!/usr/bin/env bash

export BTRFSUTIL_TESTENV="btrfsutil_testenv"

printf "%s" "Installing system dependencies"

apt-get update
apt-get install --yes \
    btrfs-progs \
    libbtrfsutil-dev \
    libbtrfsutil1 \
    gcc \
    clang \
    make

printf "%s" "Installing Rust"
su - vagrant -c bash -c "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"

cat /home/vagrant/.profile | grep -xq 'export BTRFSUTIL_TESTENV="btrfsutil_testenv"'
if [[ "$?" != "0" ]]; then
    echo 'export BTRFSUTIL_TESTENV="btrfsutil_testenv"' >> /home/vagrant/.profile
fi
