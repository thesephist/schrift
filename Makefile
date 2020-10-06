DEBUG = ./target/debug/schrift

all: tests

tests:
	cargo build
	$(DEBUG) test/000.ink
	$(DEBUG) test/002.ink
	$(DEBUG) test/003.ink
	$(DEBUG) test/004.ink
	$(DEBUG) test/005.ink
	$(DEBUG) test/006.ink
	$(DEBUG) test/007.ink
	$(DEBUG) test/008.ink
	$(DEBUG) test/009.ink
	$(DEBUG) test/010.ink
t: tests

fmt:
	inkfmt fix test/0*.ink
f: fmt

fmt-check:
	inkfmt test/0*.ink
fk: fmt-check
