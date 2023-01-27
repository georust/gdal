fn main() {
    use gdal::{Dataset, Metadata};
    use std::path::Path;

    let driver = gdal::DriverManager::get_driver_by_name("mem").unwrap();
    println!("driver description: {:?}", driver.description());

    let path = Path::new("./fixtures/tinymarble.png");
    let dataset = Dataset::open(path).unwrap();
    println!("dataset description: {:?}", dataset.description());

    let key = "INTERLEAVE";
    let domain = "IMAGE_STRUCTURE";
    let meta = dataset.metadata_item(key, domain);
    println!("domain: {domain:?} key: {key:?} -> value: {meta:?}");
}
