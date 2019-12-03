# vim: set ft=make:
#
# Author: Felix Viernickel
#   Date: 2019
#

VAR=var
ENVRC="./.envrc"

all: assets db release

.PHONY: assets
assets:
	@(ln -s ../$(ASSETS) $(ASSETS) 2>/dev/null || :)
	@(mkdir -p $(VAR)/spool 2>/dev/null || :)
	@(mkdir -p $(VAR)/store 2>/dev/null || :)

.PHONY: db
db:
	source $(ENVRC) && diesel migration run

.PHONY: service
service: assets db release
	source $(ENVRC) && cargo run --release -- run

.PHONY: release
release:
	source $(ENVRC) && cargo build --release

.PHONY: purge
purge:
	rm -rf $(VAR)/spool
	rm -rf $(VAR)/store

.PHONY: clean
clean:
	cargo clean
