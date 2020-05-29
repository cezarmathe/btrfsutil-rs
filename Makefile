PROVIDER=virtualbox

default: test

build:
	vagrant ssh -- /home/vagrant/btrfsutil_build.sh
.PHONY: build

test:
	vagrant ssh -- /home/vagrant/btrfsutil_test.sh
.PHONY: test

up:
	vagrant up --provider $(PROVIDER)
.PHONY: up

provision: up
	vagrant provision
.PHONY: provision

ssh:
	vagrant ssh
.PHONY: ssh

reload:
	vagrant reload
.PHONY: reload

halt:
	vagrant halt
.PHONY: halt
