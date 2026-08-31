.PHONY: clone hooks changelog

clone:
	./scriipts/01-clone.sh

hooks:
	./scriipts/02-setup-hooks.sh

changelog:
	git cliff -o CHANGELOG.md
