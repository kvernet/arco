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
	act -W .github/workflows/ci.yml --artifact-server-path \
		/tmp/act-artifacts --env ACT_SKIP_UPLOAD=true