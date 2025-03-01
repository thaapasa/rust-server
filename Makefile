.PHONY: check
check:
	cargo check --tests --all-features

.PHONY: clippy
clippy:
	cargo clippy --tests --all-features

.PHONY: lint
lint:
	make check
	make clippy

.PHONY: test
test:
	cargo test -p sql_macros
	cargo test -p macros
	cargo test -- --test-threads=1

.PHONY: clean
clean:
	rm -rf target

.PHONY: start
start:
	CONFIG_FILE=.dev/settings.toml cargo run main
