# Default provider for the Vagrant machine
PROVIDER=virtualbox
# Default command to be run by the wrapper
WRAPPER_CMD="cargo build"

default: wrapper

# Wraps a certain command that must be executed inside the Vagrant machine
wrapper:
	vagrant ssh -- /home/vagrant/wrapper.sh $(WRAPPER_CMD)
.PHONY: wrapper

# Run tests inside the Vagrant test environment
test:
	make wrapper WRAPPER_CMD="./scripts/btrfsutil_testenv_create.sh"
	make wrapper WRAPPER_CMD="cargo test"
	make wrapper WRAPPER_CMD="./scripts/btrfsutil_testenv_destroy.sh"
.PHONY: test

# Build the documentation the same way docs.rs would
docs:
	RUSTFLAGS="--cfg docs_rs" cargo doc --no-deps -v
.PHONY: docs

# Check the formatting of the source code
check_fmt:
	cargo fmt -- --check
.PHONY: check_fmt

# Run clippy checks
check_clippy:
	cargo clippy -- -D warnings
.PHONY: check_clippy

# Start the Vagrant machine
up:
	vagrant box update
	vagrant up --provider $(PROVIDER)
.PHONY: up

# Provision the Vagrant machine
provision: up
	vagrant provision
.PHONY: provision

# SSH inside the Vagrant machine
ssh:
	vagrant ssh
.PHONY: ssh

# Reload the Vagrant machine
reload:
	vagrant reload
.PHONY: reload

# Halt the Vagrant machine
halt:
	vagrant halt
.PHONY: halt
