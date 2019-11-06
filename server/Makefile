# vim: set ft=make:
#
# Author: Felix Viernickel
#   Date: 2019
#

VAR=var

all: build

.PHONY: assets
assets:
	@(ln -s ../$(ASSETS) $(ASSETS) 2>/dev/null || :)
	@(mkdir $(VAR)/spool 2>/dev/null || :)
	@(mkdir $(VAR)/store 2>/dev/null || :)

.PHONY: launch
launch:
	source ./.envrc && cargo run

.PHONY: build
build:
	source ./.envrc && cargo build

.PHONY: clean
clean:
	cargo clean
