.PHONY: test clippy fmt fix all

all: fmt clippy test

test:
	cargo test

clippy:
	cargo clippy -- -D warnings

fmt:
	cargo fmt --all -- --check

fix:
	cargo fmt --all

ci-act:
	act -P ubuntu-latest=catthehacker/ubuntu:full-latest \
		--artifact-server-path /tmp/act-artifacts