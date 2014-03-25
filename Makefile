RUSTC ?= rustc

rustiles: src/rustiles.rs
	mkdir -p build
	$(RUSTC) -o build/rustiles src/rustiles.rs

build/testsuite: src/rustiles.rs
	mkdir -p build
	$(RUSTC) --test -o build/testsuite src/rustiles.rs

check: build/testsuite
	./build/testsuite

clean:
	rm -rf build

.PHONY: check clean
