use gdal::raster::RasterBand;
use gdal::{Dataset, Metadata};
use std::path::Path;

fn main() {
    let path = Path::new("./fixtures/tinymarble.png");
    let dataset = Dataset::open(path).unwrap();
    println!("dataset description: {:?}", dataset.description());

    let rasterband: RasterBand = dataset.rasterband(1).unwrap();
    println!("rasterband description: {:?}", rasterband.description());
    println!("rasterband no_data_value: {:?}", rasterband.no_data_value());
    println!("rasterband type: {:?}", rasterband.band_type());
    println!("rasterband scale: {:?}", rasterband.scale());
    println!("rasterband offset: {:?}", rasterband.offset());
    if let Ok(rv) = rasterband.read_as::<u8>((20, 30), (2, 3), (2, 3), None) {
        println!("{:?}", rv.data);
    }
}
