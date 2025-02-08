.PHONY: check
check:
	cargo check

.PHONY: clippy
clippy:
	cargo clippy

.PHONY: lint
lint:
	make check
	make clippy

.PHONY: start
start:
	CONFIG_FILE=.dev/settings.toml cargo run main
