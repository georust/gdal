use std::path::Path;
use super::{ByteBuffer, Driver, Dataset};
use super::gdal_enums::{GDALDataType};
use metadata::Metadata;


macro_rules! fixture {
    ($name:expr) => (
        Path::new(file!())
            .parent().unwrap()
            .parent().unwrap()
            .parent().unwrap()
            .join("fixtures").as_path()
            .join($name).as_path()
    )
}


#[test]
fn test_open() {
    let dataset = Dataset::open(fixture!("tinymarble.png"));
    assert!(dataset.is_some());

    let missing_dataset = Dataset::open(fixture!("no_such_file.png"));
    assert!(missing_dataset.is_none());
}


#[test]
fn test_get_raster_size() {
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let (size_x, size_y) = dataset.size();
    assert_eq!(size_x, 100);
    assert_eq!(size_y, 50);
}


#[test]
fn test_get_raster_count() {
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let count = dataset.count();
    assert_eq!(count, 3);
}


#[test]
fn test_get_projection() {
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    //dataset.set_projection("WGS84");
    let projection = dataset.projection();
    assert_eq!(projection.chars().take(16).collect::<String>(), "GEOGCS[\"WGS 84\",");
}


#[test]
fn test_read_raster() {
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let rv = dataset.read_raster(
        1,
        (20, 30),
        (2, 3),
        (2, 3)
    ).unwrap();
    assert_eq!(rv.size.0, 2);
    assert_eq!(rv.size.1, 3);
    assert_eq!(rv.data, vec!(7, 7, 7, 10, 8, 12));
}


#[test]
fn test_write_raster() {
    let driver = Driver::get("MEM").unwrap();
    let dataset = driver.create("", 20, 10, 1).unwrap();

    // create a 2x1 raster
    let raster = ByteBuffer{
        size: (2, 1),
        data: vec!(50u8, 20u8)
    };

    // epand it to fill the image (20x10)
    dataset.write_raster(
        1,
        (0, 0),
        (20, 10),
        raster
    );

    // read a pixel from the left side
    let left = dataset.read_raster(
        1,
        (5, 5),
        (1, 1),
        (1, 1)
    ).unwrap();
    assert_eq!(left.data[0], 50u8);

    // read a pixel from the right side
    let right = dataset.read_raster(
        1,
        (15, 5),
        (1, 1),
        (1, 1)
    ).unwrap();
    assert_eq!(right.data[0], 20u8);
}


#[test]
fn test_get_dataset_driver() {
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let driver = dataset.driver();
    assert_eq!(driver.short_name(), "PNG");
    assert_eq!(driver.long_name(), "Portable Network Graphics");
}

#[test]
fn test_get_description() {

    let driver = Driver::get("mem").unwrap();
    assert_eq!(driver.description(), Some("MEM".to_owned()));
}

#[test]
fn test_get_metadata_item() {
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let key = "None";
    let domain = "None";
    let meta = dataset.metadata_item(key, domain);
    assert_eq!(meta, None);

    let key = "INTERLEAVE";
    let domain = "IMAGE_STRUCTURE";
    let meta = dataset.metadata_item(key, domain);
    assert_eq!(meta, Some(String::from("PIXEL")));
}

#[test]
fn test_set_metadata_item() {
    let driver = Driver::get("MEM").unwrap();
    let mut dataset = driver.create("", 1, 1, 1).unwrap();

    let key = "Test_Key";
    let domain = "Test_Domain";
    let value = "Test_Value";
    let result = dataset.set_metadata_item(key, value, domain);
    assert_eq!(result, Ok(()));

    let result = dataset.metadata_item(key, domain);
    assert_eq!(Some(value.to_owned()), result);
}

#[test]
fn test_create() {
    let driver = Driver::get("MEM").unwrap();
    let dataset = driver.create("", 10, 20, 3).unwrap();
    assert_eq!(dataset.size(), (10, 20));
    assert_eq!(dataset.count(), 3);
    assert_eq!(dataset.driver().short_name(), "MEM");
}

#[test]
fn test_create_with_band_type() {
    let driver = Driver::get("MEM").unwrap();
    let dataset = driver.create_with_band_type::<f32>("", 10, 20, 3).unwrap();
    assert_eq!(dataset.size(), (10, 20));
    assert_eq!(dataset.count(), 3);
    assert_eq!(dataset.driver().short_name(), "MEM");
    assert_eq!(dataset.band_type(1), Some(GDALDataType::GDT_Float32))
}

#[test]
fn test_create_copy() {
    let driver = Driver::get("MEM").unwrap();
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let copy = dataset.create_copy(driver, "").unwrap();
    assert_eq!(copy.size(), (100, 50));
    assert_eq!(copy.count(), 3);
}


#[test]
fn test_geo_transform() {
    let driver = Driver::get("MEM").unwrap();
    let dataset = driver.create("", 20, 10, 1).unwrap();
    let transform = [0., 1., 0., 0., 0., 1.];
    dataset.set_geo_transform(&transform);
    assert_eq!(dataset.geo_transform(), Some(transform));
}


#[test]
fn test_get_driver_by_name() {
    let missing_driver = Driver::get("wtf");
    assert!(missing_driver.is_none());

    let ok_driver = Driver::get("GTiff");
    assert!(ok_driver.is_some());
    let driver = ok_driver.unwrap();
    assert_eq!(driver.short_name(), "GTiff");
    assert_eq!(driver.long_name(), "GeoTIFF");
}

#[test]
fn test_read_raster_as() {
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let rv = dataset.read_raster_as::<u8>(
        1,
        (20, 30),
        (2, 3),
        (2, 3)
    ).unwrap();
    assert_eq!(rv.data, vec!(7, 7, 7, 10, 8, 12));
    assert_eq!(rv.size.0, 2);
    assert_eq!(rv.size.1, 3);
    assert_eq!(dataset.band_type(1), Some(GDALDataType::GDT_Byte));
}

#[test]
fn test_read_full_raster_as() {
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let rv = dataset.read_full_raster_as::<u8>(1).unwrap();
    assert_eq!(rv.size.0, 100);
    assert_eq!(rv.size.1, 50);
}

#[test]
fn test_get_band_type() {
    let driver = Driver::get("MEM").unwrap();
    let dataset = driver.create("", 20, 10, 1).unwrap();
    assert_eq!(dataset.band_type(1), Some(GDALDataType::GDT_Byte));
    assert_eq!(dataset.band_type(2), None);
}

#[test]
fn test_get_rasterband() {
    let driver = Driver::get("MEM").unwrap();
    let dataset = driver.create("", 20, 10, 1).unwrap();
    let rasterband = dataset.rasterband(1);
    assert!(rasterband.is_some())
}

#[test]
fn test_get_no_data_value() {
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let rasterband = dataset.rasterband(1).unwrap();
    let no_data_value = rasterband.no_data_value();
    assert!(no_data_value.is_none());

    // let dataset = Dataset::open(fixture!("bluemarble.tif")).unwrap();
    // let rasterband = dataset.get_rasterband(1).unwrap();
    // let no_data_value = rasterband.get_no_data_value();
    // assert_eq!(no_data_value, Some(0.0));
}

#[test]
fn test_get_scale() {
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let rasterband = dataset.rasterband(1).unwrap();
    let scale = rasterband.scale();
    assert_eq!(scale, Some(1.0));
}

#[test]
fn test_get_offset() {
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let rasterband = dataset.rasterband(1).unwrap();
    let offset = rasterband.offset();
    assert_eq!(offset, Some(0.0));
}
