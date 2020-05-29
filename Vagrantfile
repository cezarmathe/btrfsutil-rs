# -*- mode: ruby -*-
# vi: set ft=ruby :

Vagrant.configure("2") do |config|
  config.vm.box = "generic/ubuntu1910"

  config.vm.synced_folder "./target", "/home/vagrant/btrfsutil-rs/target", mount_options: ["rw"]
  config.vm.synced_folder  ".", "/home/vagrant/btrfsutil-rs", mount_options: ["ro"]

  config.vm.provision "shell", path: "./scripts/provision.sh"
  config.vm.provision "file", source: "./scripts/btrfsutil_testenv_bootstrap.sh", destination: "/home/vagrant/btrfsutil_testenv_bootstrap.sh"
  config.vm.provision "file", source: "./scripts/btrfsutil_testenv_teardown.sh", destination: "/home/vagrant/btrfsutil_testenv_teardown.sh"
  config.vm.provision "file", source: "./scripts/btrfsutil_test.sh", destination: "/home/vagrant/btrfsutil_test.sh"
  config.vm.provision "file", source: "./scripts/btrfsutil_build.sh", destination: "/home/vagrant/btrfsutil_build.sh"
end