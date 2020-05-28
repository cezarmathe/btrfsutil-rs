PROVIDER=virtualbox
TARGET=debug

default: test

up:
	cd vagrant; vagrant up --provider $(PROVIDER)
.PHONY: up

provision:
	cd vagrant; vagrant provision
.PHONY: provision

ssh:
	cd vagrant; vagrant ssh
.PHONY: ssh

reload:
	cd vagrant; vagrant reload
.PHONY: reload

halt:
	cd vagrant; vagrant halt
.PHONY: halt

test: up provision
	cargo build --tests
	cd vagrant; vagrant ssh -- /home/vagrant/btrfsutil_test.sh "$(TARGET)"
