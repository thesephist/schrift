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
t: tests

fmt:
	inkfmt fix test/*.ink
f: fmt

fmt-check:
	inkfmt test/*.ink
fk: fmt-check
