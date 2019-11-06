# vim: set ft=make:
#
# Author: Felix Viernickel
#   Date: 2019
#

COPY=rsync -avzh
SYNC=rsync -avzh --delete
LIB=lib
SRC=src

APP=app
IMG=img
INDEX=index
DST=assets
ASSETS=assets

all: app css

.PHONY: assets
assets:
	@(ln -s ../$(ASSETS) $(ASSETS) 2>/dev/null || :)

.PHONY: css-notify
css-notify:
	@compass watch --config compass.config.rb

.PHONY: css
css:
	@compass compile --config compass.config.rb

.PHONY: app-notify
app-notify:
	@while F=$$(inotifywait -e CREATE -e MODIFY --format %f $(SRC)); do $(MAKE) app; done

.PHONY: app
app:
	@$(COPY)  $(SRC)/$(INDEX)/ $(DST)/
	@$(SYNC)  $(SRC)/$(APP)/   $(DST)/$(APP)/
	@$(SYNC)  $(SRC)/$(IMG)/   $(DST)/$(IMG)/
	@$(SYNC)  $(LIB)/          $(DST)/$(LIB)/

.PHONY: clean
clean:
	@rm $(ASSETS)/* -rf
	@touch $(ASSETS)/.gitkeep
