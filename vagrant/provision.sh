#!/usr/bin/env bash

export BTRFSUTIL_TESTENV="btrfsutil_testenv"

printf "%s" "Installing system dependencies"

apt-get update
apt-get install --yes \
    btrfs-progs \
    libbtrfsutil-dev \
    libbtrfsutil1 \
    gcc \
    clang

# printf "%s" "Installing Rust"
# su - vagrant -c bash -c "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y"

# printf "%s" "Setting up the test environment"
# if [[ ! -f "/home/vagrant/${BTRFSUTIL_TESTENV}.img" ]]; then
#     su - vagrant -c bash -c "fallocate -l 1G ~/${BTRFSUTIL_TESTENV}.img"
#     su - vagrant -c bash -c "mkfs -t btrfs -q ~/${BTRFSUTIL_TESTENV}.img"
# fi
# if [[ ! -d "/mnt/${BTRFSUTIL_TESTENV}" ]]; then
#     mkdir "/mnt/${BTRFSUTIL_TESTENV}"
# fi
# mount -t btrfs -o loop "/home/vagrant/${BTRFSUTIL_TESTENV}.img" "/mnt/${BTRFSUTIL_TESTENV}"

cat /home/vagrant/.profile | grep -xq 'export BTRFSUTIL_TESTENV="btrfsutil_testenv"'
if [[ "$?" != "0" ]]; then
    echo 'export BTRFSUTIL_TESTENV="btrfsutil_testenv"' >> /home/vagrant/.profile
fi
