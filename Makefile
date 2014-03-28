RUSTC ?= rustc
RUSTFLAGS ?=

src_files=\
	        src/rustiles.rs \
	        src/gdal.rs

all: tile rustiles

tile: src/tile.rs src/gdal.rs
	mkdir -p build
	$(RUSTC) $(RUSTFLAGS) -o build/tile src/tile.rs

rustiles: $(src_files)
	mkdir -p build
	$(RUSTC) $(RUSTFLAGS) -o build/rustiles src/rustiles.rs

build/testsuite: $(src_files)
	mkdir -p build
	$(RUSTC) $(RUSTFLAGS) -A dead_code --test -o build/testsuite src/rustiles.rs

check: build/testsuite
	RUSTILES_TEST_FIXTURES=`pwd`/fixtures ./build/testsuite

clean:
	rm -rf build

.PHONY: check clean
