extern crate gdal;

fn main() {
    use std::path::Path;
    use gdal::raster::dataset::Dataset;
    use gdal::metadata::Metadata;

    let path = Path::new("./fixtures/tinymarble.png");
    let dataset = Dataset::open(path).unwrap();
    let key = "INTERLEAVE";
    let domain = "IMAGE_STRUCTURE";
    let meta = dataset.get_metadata_item(key, domain);
    println!("domain: {:?} key: {:?} -> value: {:?}", domain, key, meta);

}
