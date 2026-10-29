SHELL := /bin/bash -eu -o pipefail
.PHONY: setup check-version package build test clean
setup:
	python3 package.py prepare
check-version:
	python3 package.py check
package: check-version
	python3 package.py package
build: check-version
	cargo build --locked
test: check-version
	CARGO_TARGET_DIR="$(CURDIR)/target" cargo test --manifest-path testcrate/Cargo.toml --locked -- --test-threads=1
clean:
	rm -rf target dist testcrate/target mnn-sources.tar.xz
