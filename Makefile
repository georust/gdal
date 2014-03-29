RUSTC ?= rustc
RUSTFLAGS ?=

src_files=\
	src/rustiles.rs \
	src/gdal/mod.rs \
	src/gdal/driver.rs \
	src/gdal/dataset.rs \
	src/gdal/proj.rs \
	src/tile.rs

all: build/tile build/rustiles

build/tile: $(src_files)
	mkdir -p build
	$(RUSTC) $(RUSTFLAGS) -o build/tile src/tile.rs

build/rustiles: $(src_files)
	mkdir -p build
	$(RUSTC) $(RUSTFLAGS) -o build/rustiles src/rustiles.rs

build/testsuite: $(src_files)
	mkdir -p build
	$(RUSTC) $(RUSTFLAGS) -A dead_code --test -o build/testsuite src/rustiles.rs

check: build/testsuite
	RUSTILES_TEST_FIXTURES=`pwd`/fixtures ./build/testsuite

clean:
	rm -rf build

.PHONY: all check clean
