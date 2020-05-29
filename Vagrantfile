# -*- mode: ruby -*-
# vi: set ft=ruby :

Vagrant.configure("2") do |config|
  config.vm.box = "generic/ubuntu1910"

  config.vm.synced_folder "./target", "/home/vagrant/btrfsutil-rs/target", mount_options: ["rw"]
  config.vm.synced_folder  ".", "/home/vagrant/btrfsutil-rs", mount_options: ["ro"]

  config.vm.provision "shell", path: "./scripts/provision.sh"
  config.vm.provision "file", source: "./scripts/wrapper.sh", destination: "/home/vagrant/wrapper.sh"
end
