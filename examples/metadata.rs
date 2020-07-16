
fn main() {
    use gdal::metadata::Metadata;
    use gdal::dataset::{Dataset, DatasetCommon};
    use gdal::driver::{DriverCommon, Driver};
    use std::path::Path;

    let driver = Driver::get("mem").unwrap();
    println!("driver description: {:?}", driver.description());

    let path = Path::new("./fixtures/tinymarble.png");
    let dataset = Dataset::open(path).unwrap();
    println!("dataset description: {:?}", dataset.description());

    let key = "INTERLEAVE";
    let domain = "IMAGE_STRUCTURE";
    let meta = dataset.metadata_item(key, domain);
    println!("domain: {:?} key: {:?} -> value: {:?}", domain, key, meta);
}
