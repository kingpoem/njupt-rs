.PHONY: clone hooks changelog

clone:
	./scripts/01-clone.sh

hooks:
	./scripts/02-setup-hooks.sh

changelog:
	git cliff -o CHANGELOG.md
