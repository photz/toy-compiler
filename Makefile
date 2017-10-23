
.PHONY: build watch

watch:
	(find src -name '*.rs'; echo Makefile; ls *.php) \
	| grep -v '#' \
	| entr -r make build

build:
	cargo run test.php

