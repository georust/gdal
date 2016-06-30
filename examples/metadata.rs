extern crate gdal;

fn main() {
    use std::path::Path;
    use gdal::raster::dataset::Dataset;
    use gdal::metadata::Metadata;

    let driver = gdal::raster::driver::Driver::get("mem").unwrap();
    println!("driver description: {:?}", driver.description());

    let path = Path::new("./fixtures/tinymarble.png");
    let dataset = Dataset::open(path).unwrap();
    println!("dataset description: {:?}", dataset.description());

    let key = "INTERLEAVE";
    let domain = "IMAGE_STRUCTURE";
    let meta = dataset.metadata_item(key, domain);
    println!("domain: {:?} key: {:?} -> value: {:?}", domain, key, meta);

}
