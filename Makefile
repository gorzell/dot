TARGETS=armv7-unknown-linux-gnueabihf aarch64-unknown-linux-gnu riscv64gc-unknown-linux-gnu x86_64-unknown-linux-gnu
# TODO: Can only compile for apple on OSX
# x86_64-apple-darwin

.PHONY: all
all: format build lint test

.PHONY: bootstrap
bootstrap:
	rustup update
	cargo install cross
	cargo install cargo-deb

.PHONY: clean
clean:
	cargo clean

.PHONY: format
format:
	RUST_LOG=error; cargo fmt

.PHONY: lint
lint:
	cargo fmt --all -- --check
	# Sadly the clippy team can't seem to figure out how to allow enabling/disabling lints in their config file (https://github.com/rust-lang/cargo/issues/5034)
	# So we have to do it with CLI flags.
	cargo clippy -- --deny warnings #-D clippy::unwrap_used
	cargo clippy --tests -- --deny warnings -A clippy::unwrap_used

.PHONY: build
build:
	cargo build

.PHONY: test
test:
	# Use --all-targets to ensure that all of the benchmarks compile.
	RUST_BACKTRACE=1 cargo test --all-targets
	# Amazingly, `--all-targets` causes doc-tests not to run.
	RUST_BACKTRACE=1 cargo test --doc

.PHONY: cross
cross:
	for target in $(TARGETS); do \
		cross build --target $$target ; \
	done

.PHONY: cross-test
cross-test:
	for target in $(TARGETS); do \
		cross test --target $$target ; \
	done

.PHONY: cross-release
cross-release:
	set -ex && \
	for target in $(TARGETS); do \
		cross build --release --target $$target ; \
	done

.PHONY: install
install:
	cargo install

.PHONY: build-release
build-release:
	cargo build --release

.PHONY: deb
deb:
	for target in $(TARGETS); do \
		cargo deb --no-build --no-strip --target $$target ; \
	done
