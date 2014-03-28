extern crate sync;

use std::os::args;
use std::path::Path;
use std::io::{File, TempDir, stdio};

#[allow(dead_code)]
mod gdal;


fn main() {
    let memory_driver = gdal::get_driver("MEM").unwrap();
    let png_driver = gdal::get_driver("PNG").unwrap();

    let source = gdal::open(&Path::new(args()[1])).unwrap();
    println!(
        "size: {}, bands: {}",
        source.get_raster_size(),
        source.get_raster_count()
    );

    let tile = memory_driver.create("", 256, 256, 3).unwrap();
    for band in range(1, 4) {
        let raster = source.read_raster(band, 10000, 1600, 3000, 2000, 256, 256);
        tile.write_raster(band, 0, 0, 256, 256, raster);
    }

    let tmp = TempDir::new("rustile").unwrap();
    let tile_path = tmp.path().join("tile.png");
    tile.create_copy(png_driver, tile_path.as_str().unwrap());
    let tile_data = File::open(&tile_path).read_to_end().unwrap();
    assert!(stdio::stdout_raw().write(tile_data).is_ok());
}
