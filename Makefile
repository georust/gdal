RUSTC ?= rustc
RUSTFLAGS ?=

src_files=\
	        src/rustiles.rs \
	        src/gdal.rs

rustiles: $(src_files)
	mkdir -p build
	$(RUSTC) $(RUSTFLAGS) -o build/rustiles src/rustiles.rs

build/testsuite: $(src_files)
	mkdir -p build
	$(RUSTC) $(RUSTFLAGS) -A dead_code --test -o build/testsuite src/rustiles.rs

check: build/testsuite
	./build/testsuite

clean:
	rm -rf build

.PHONY: check clean
