RUSTC ?= rustc
RUSTFLAGS ?=

src_files=\
	src/gdal/lib.rs \
	src/gdal/driver.rs \
	src/gdal/raster.rs \
	src/gdal/proj.rs \
	src/gdal/geom.rs

all: libgdal

libgdal: $(src_files)
	mkdir -p build
	$(RUSTC) $(RUSTFLAGS) --out-dir=build src/gdal/lib.rs

build/testsuite: $(src_files)
	mkdir -p build
	$(RUSTC) $(RUSTFLAGS) -A dead_code --test -o build/testsuite src/gdal/lib.rs

check: build/testsuite
	RUST_GDAL_TEST_FIXTURES=`pwd`/fixtures ./build/testsuite

clean:
	rm -rf build

.PHONY: all check clean
