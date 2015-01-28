use std::path::Path;
use super::super::geom::Point;
use super::{ByteBuffer, driver, open};


fn fixtures() -> Path {
    return Path::new(file!()).dir_path().dir_path().dir_path().join("fixtures");
}


#[test]
fn test_open() {
    let dataset = open(&fixtures().join("tinymarble.png"));
    assert!(dataset.is_some());

    let missing_dataset = open(&fixtures().join("no_such_file.png"));
    assert!(missing_dataset.is_none());
}


#[test]
fn test_get_raster_size() {
    let dataset = open(&fixtures().join("tinymarble.png")).unwrap();
    let (size_x, size_y) = dataset.size();
    assert_eq!(size_x, 100);
    assert_eq!(size_y, 50);
}


#[test]
fn test_get_raster_count() {
    let dataset = open(&fixtures().join("tinymarble.png")).unwrap();
    let count = dataset.count();
    assert_eq!(count, 3);
}


#[test]
fn test_get_projection() {
    let dataset = open(&fixtures().join("tinymarble.png")).unwrap();
    //dataset.set_projection("WGS84");
    let projection = dataset.projection();
    assert_eq!(projection.as_slice().slice(0, 16), "GEOGCS[\"WGS 84\",");
}


#[test]
fn test_read_raster() {
    let dataset = open(&fixtures().join("tinymarble.png")).unwrap();
    let rv = dataset.read_raster(
        1,
        Point::new(20, 30),
        Point::new(2, 3),
        Point::new(2, 3)
    );
    assert_eq!(rv.size.x, 2);
    assert_eq!(rv.size.y, 3);
    assert_eq!(rv.data, vec!(7, 7, 7, 10, 8, 12));
}


#[test]
fn test_write_raster() {
    let driver = driver("MEM").unwrap();
    let dataset = driver.create("", 20, 10, 1).unwrap();

    // create a 2x1 raster
    let raster = ByteBuffer{
        size: Point::new(2, 1),
        data: vec!(50u8, 20u8)
    };

    // epand it to fill the image (20x10)
    dataset.write_raster(
        1,
        Point::new(0, 0),
        Point::new(20, 10),
        raster
    );

    // read a pixel from the left side
    let left = dataset.read_raster(
        1,
        Point::new(5, 5),
        Point::new(1, 1),
        Point::new(1, 1)
    );
    assert_eq!(left.data[0], 50u8);

    // read a pixel from the right side
    let right = dataset.read_raster(
        1,
        Point::new(15, 5),
        Point::new(1, 1),
        Point::new(1, 1)
    );
    assert_eq!(right.data[0], 20u8);
}


#[test]
fn test_get_dataset_driver() {
    let dataset = open(&fixtures().join("tinymarble.png")).unwrap();
    let driver = dataset.driver();
    assert_eq!(driver.short_name().as_slice(), "PNG");
    assert_eq!(driver.long_name().as_slice(), "Portable Network Graphics");
}


#[test]
fn test_create() {
    let driver = driver("MEM").unwrap();
    let dataset = driver.create("", 10, 20, 3).unwrap();
    assert_eq!(dataset.size(), (10, 20));
    assert_eq!(dataset.count(), 3);
    assert_eq!(dataset.driver().short_name().as_slice(), "MEM");
}


#[test]
fn test_create_copy() {
    let driver = driver("MEM").unwrap();
    let dataset = open(&fixtures().join("tinymarble.png")).unwrap();
    let copy = dataset.create_copy(driver, "").unwrap();
    assert_eq!(copy.size(), (100, 50));
    assert_eq!(copy.count(), 3);
}


#[test]
fn test_geo_transform() {
    let driver = driver("MEM").unwrap();
    let dataset = driver.create("", 20, 10, 1).unwrap();
    let transform = vec!(0., 1., 0., 0., 0., 1.);
    dataset.set_geo_transform(transform.as_slice());
    assert_eq!(dataset.geo_transform(), transform);
}


#[test]
fn test_get_driver_by_name() {
    let missing_driver = driver("wtf");
    assert!(missing_driver.is_none());

    let ok_driver = driver("GTiff");
    assert!(ok_driver.is_some());
    let driver = ok_driver.unwrap();
    assert_eq!(driver.short_name().as_slice(), "GTiff");
    assert_eq!(driver.long_name().as_slice(), "GeoTIFF");
}
