# vim: set ft=make:
#
# Author: Felix Viernickel
#   Date: 2019
#

CLIENT=client
SERVER=server
ASSETS=assets

all: client server

.PHONY: assets
assets:
	@$(MAKE) -C $(CLIENT) assets
	@$(MAKE) -C $(SERVER) assets

.PHONY: client
client:
	@$(MAKE) -C $(CLIENT)

.PHONY: server
server:
	@$(MAKE) -C $(SERVER)

.PHONY: clean
clean:
	@$(MAKE) -C $(SERVER) clean
	@$(MAKE) -C $(CLIENT) clean

.PHONY: service
service:
	@$(MAKE) -C $(SERVER) service
