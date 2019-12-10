# vim: set ft=make:

VAR="test"
ENVRC="./.envrc"

all: assets db release

.PHONY: assets
assets:
	@(ln -s ../$(ASSETS) $(ASSETS) 2>/dev/null || :)
	@(mkdir -p $(VAR)/spool 2>/dev/null || :)
	@(mkdir -p $(VAR)/store 2>/dev/null || :)

.PHONY: schema-patch
schema-patch:
	sed -i src/db/schema.rs -e "10s/.*/        bytes -> BigInt,/"
	cargo fmt -- src/db/schema.rs

.PHONY: db
db:
	source $(ENVRC) && diesel migration run
	$(MAKE) schema-patch

.PHONY: service
service: assets db release
	source $(ENVRC) && cargo run --release -- run

.PHONY: debug
debug:
	source $(ENVRC) && cargo build --debug

.PHONY: release
release:
	source $(ENVRC) && cargo build --release

.PHONY: purge
purge:
	$(MAKE) clean
	rm -rf $(VAR)/spool
	rm -rf $(VAR)/store

.PHONY: clean
clean:
	cargo clean
