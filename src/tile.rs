extern crate sync;
extern crate geom;

use std::os::args;
use std::path::Path;
use std::io::{File, TempDir, stdio};
use geom::point::Point2D;
use gdal::proj::{Proj, DEG_TO_RAD};
use gdal::dataset::{Dataset, open};
use gdal::driver::get_driver;

static webmerc_limit: f64 = 20037508.342789244;


fn as_point((x, y): (f64, f64)) -> Point2D<f64> {
    return Point2D(x, y);
}


fn mul<T:Clone + Mul<T,T>>(value: &Point2D<T>, factor: T) -> Point2D<T> {
    return Point2D(value.x * factor, value.y * factor);
}


pub fn tile(source: Dataset, (x, y, z): (int, int, int)) -> ~[u8] {
    let memory_driver = get_driver("MEM").unwrap();
    let png_driver = get_driver("PNG").unwrap();

    let wgs84 = Proj::new("+proj=longlat +datum=WGS84 +no_defs").unwrap();
    let webmerc = Proj::new(
        "+proj=merc +a=6378137 +b=6378137 +lat_ts=0.0 +lon_0=0.0 +x_0=0.0 " +
        "+y_0=0 +k=1.0 +units=m +nadgrids=@null +wktext  +no_defs").unwrap();

    let tile = Point2D(x, y);
    let tile_size = (webmerc_limit * 4.) / ((2 << z) as f64);
    let tile_min = Point2D(
        tile_size * (tile.x as f64) - webmerc_limit,
        webmerc_limit - tile_size * (tile.y as f64));
    let tile_max = tile_min + Point2D(tile_size, -tile_size);
    let nw = mul(&as_point(webmerc.project(&wgs84, tile_min.x, tile_min.y)), 1./DEG_TO_RAD);
    let se = mul(&as_point(webmerc.project(&wgs84, tile_max.x, tile_max.y)), 1./DEG_TO_RAD);

    let (width, height) = source.get_raster_size();
    let source_bounds = Point2D(width as f64, height as f64);
    assert!(stdio::stderr().write(format!(
        "size: {}, bands: {}\n",
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
    return File::open(&tile_path).read_to_end().unwrap();
}


fn main() {
    let source = open(&Path::new(args()[1])).unwrap();
    let tile_data = tile(source, (8, 5, 4));
    assert!(stdio::stdout_raw().write(tile_data).is_ok());
}
