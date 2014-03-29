extern crate sync;
extern crate geom;

use std::os::args;
use std::path::Path;
use std::io::{File, TempDir, stdio};
use geom::point::Point2D;

#[allow(dead_code)]
mod gdal;


fn main() {
    let memory_driver = gdal::driver::get_driver("MEM").unwrap();
    let png_driver = gdal::driver::get_driver("PNG").unwrap();

    let source = gdal::dataset::open(&Path::new(args()[1])).unwrap();
    let (width, height) = source.get_raster_size();
    let source_bounds = Point2D(width as f64, height as f64);
    assert!(stdio::stderr().write(format!(
        "size: {}, bands: {}",
        (width, height),
        source.get_raster_count()
    ).as_bytes()).is_ok());

    fn xy(lng_lat: &Point2D<f64>, source_bounds: &Point2D<f64>) -> Point2D<f64> {
        let x = (lng_lat.x + 180.) / 360. * source_bounds.x;
        let y = (90. - lng_lat.y) / 180. * source_bounds.y;
        return Point2D(x, y);
    }

    let tile = memory_driver.create("", 256, 256, 3).unwrap();
    for band in range(1, 4) {
        let nw: Point2D<f64> = Point2D(-13., 64.);
        let se: Point2D<f64> = Point2D(37., 30.);

        let xy_min = xy(&nw, &source_bounds);
        let xy_max = xy(&se, &source_bounds);
        let xy_bounds = xy_max - xy_min;

        let raster = source.read_raster(
            band,
            xy_min.x as int,
            xy_min.y as int,
            xy_bounds.x as uint,
            xy_bounds.y as uint,
            256,
            256
        );
        tile.write_raster(band, 0, 0, 256, 256, raster);
    }

    let tmp = TempDir::new("rustile").unwrap();
    let tile_path = tmp.path().join("tile.png");
    tile.create_copy(png_driver, tile_path.as_str().unwrap());
    let tile_data = File::open(&tile_path).read_to_end().unwrap();
    assert!(stdio::stdout_raw().write(tile_data).is_ok());
}
