extern crate sync;

#[allow(dead_code)]
mod gdal;


fn main() {
    let memory_driver = gdal::get_driver("MEM").unwrap();

    println!("hello tile! {}", memory_driver.get_short_name());
}
