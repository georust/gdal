use crate::dataset::Dataset;
use crate::metadata::Metadata;
use crate::raster::ByteBuffer;
use crate::Driver;
use gdal_sys::GDALDataType;
use std::path::Path;

#[cfg(feature = "ndarray")]
use ndarray::arr2;

macro_rules! fixture {
    ($name:expr) => {
        Path::new(file!())
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("fixtures")
            .as_path()
            .join($name)
            .as_path()
    };
}

#[test]
fn test_open() {
    let dataset = Dataset::open(fixture!("tinymarble.png"));
    assert!(dataset.is_ok());

    let missing_dataset = Dataset::open(fixture!("no_such_file.png"));
    assert!(missing_dataset.is_err());
}

#[test]
fn test_get_raster_size() {
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let (size_x, size_y) = dataset.raster_size();
    assert_eq!(size_x, 100);
    assert_eq!(size_y, 50);
}

#[test]
fn test_get_raster_block_size() {
    let band_index = 1;
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let rasterband = dataset.rasterband(band_index).unwrap();
    let (size_x, size_y) = rasterband.block_size();
    assert_eq!(size_x, 100);
    assert_eq!(size_y, 1);
}

#[test]
fn test_get_raster_count() {
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let count = dataset.raster_count();
    assert_eq!(count, 3);
}

#[test]
fn test_get_projection() {
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    //dataset.set_projection("WGS84");
    let projection = dataset.projection();
    assert_eq!(
        projection.chars().take(16).collect::<String>(),
        "GEOGCS[\"WGS 84\","
    );
}

#[test]
fn test_read_raster() {
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let rb = dataset.rasterband(1).unwrap();
    let rv = rb.read_as::<u8>((20, 30), (2, 3), (2, 3)).unwrap();
    assert_eq!(rv.size.0, 2);
    assert_eq!(rv.size.1, 3);
    assert_eq!(rv.data, vec!(7, 7, 7, 10, 8, 12));

    let mut buf = rv;
    rb.read_into_slice((20, 30), (2, 3), (2, 3), &mut buf.data)
        .unwrap();
    assert_eq!(buf.data, vec!(7, 7, 7, 10, 8, 12));
}

#[test]
fn test_write_raster() {
    let driver = Driver::get("MEM").unwrap();
    let dataset = driver.create("", 20, 10, 1).unwrap();

    // create a 2x1 raster
    let raster = ByteBuffer {
        size: (2, 1),
        data: vec![50u8, 20u8],
    };

    // epand it to fill the image (20x10)
    let rb = dataset.rasterband(1).unwrap();

    let res = rb.write((0, 0), (20, 10), &raster);

    assert!(res.is_ok());

    // read a pixel from the left side
    let left = rb.read_as::<u8>((5, 5), (1, 1), (1, 1)).unwrap();
    assert_eq!(left.data[0], 50u8);

    // read a pixel from the right side
    let right = rb.read_as::<u8>((15, 5), (1, 1), (1, 1)).unwrap();
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
    assert_eq!(driver.description().unwrap(), "MEM".to_string());
}

#[test]
fn test_get_metadata_domains() {
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let mut domains = dataset.metadata_domains();
    if domains[0].is_empty() {
        domains.remove(0);
    }

    assert_eq!(
        domains,
        vec!(
            "IMAGE_STRUCTURE",
            "xml:XMP",
            "DERIVED_SUBDATASETS",
            "COLOR_PROFILE"
        )
    );
}

#[test]
fn test_get_metadata_domain() {
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let domain = "None";
    let meta = dataset.metadata_domain(domain);
    assert_eq!(meta, None);

    let domain = "IMAGE_STRUCTURE";
    let meta = dataset.metadata_domain(domain);
    assert_eq!(meta, Some(vec!(String::from("INTERLEAVE=PIXEL"))));
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
    assert!(result.is_ok());

    let result = dataset.metadata_item(key, domain);
    assert_eq!(Some(value.to_owned()), result);
}

#[test]
fn test_create() {
    let driver = Driver::get("MEM").unwrap();
    let dataset = driver.create("", 10, 20, 3).unwrap();
    assert_eq!(dataset.raster_size(), (10, 20));
    assert_eq!(dataset.raster_count(), 3);
    assert_eq!(dataset.driver().short_name(), "MEM");
}

#[test]
fn test_create_with_band_type() {
    let driver = Driver::get("MEM").unwrap();
    let dataset = driver.create_with_band_type::<f32>("", 10, 20, 3).unwrap();
    assert_eq!(dataset.raster_size(), (10, 20));
    assert_eq!(dataset.raster_count(), 3);
    assert_eq!(dataset.driver().short_name(), "MEM");
    let rb = dataset.rasterband(1).unwrap();
    assert_eq!(rb.band_type(), GDALDataType::GDT_Float32)
}

#[test]
fn test_create_copy() {
    let driver = Driver::get("MEM").unwrap();
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let copy = dataset.create_copy(&driver, "").unwrap();
    assert_eq!(copy.raster_size(), (100, 50));
    assert_eq!(copy.raster_count(), 3);
}

#[test]
#[allow(clippy::float_cmp)]
fn test_geo_transform() {
    let driver = Driver::get("MEM").unwrap();
    let dataset = driver.create("", 20, 10, 1).unwrap();
    let transform = [0., 1., 0., 0., 0., 1.];
    assert!(dataset.set_geo_transform(&transform).is_ok());
    assert_eq!(dataset.geo_transform().unwrap(), transform);
}

#[test]
fn test_get_driver_by_name() {
    let missing_driver = Driver::get("wtf");
    assert!(missing_driver.is_err());

    let ok_driver = Driver::get("GTiff");
    assert!(ok_driver.is_ok());
    let driver = ok_driver.unwrap();
    assert_eq!(driver.short_name(), "GTiff");
    assert_eq!(driver.long_name(), "GeoTIFF");
}

#[test]
fn test_read_raster_as() {
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let rb = dataset.rasterband(1).unwrap();
    let rv = rb.read_as::<u8>((20, 30), (2, 3), (2, 3)).unwrap();
    assert_eq!(rv.data, vec!(7, 7, 7, 10, 8, 12));
    assert_eq!(rv.size.0, 2);
    assert_eq!(rv.size.1, 3);
    assert_eq!(rb.band_type(), GDALDataType::GDT_Byte);
}

#[test]
#[cfg(feature = "ndarray")]
fn test_read_raster_as_array() {
    let band_index = 1;
    let (left, top) = (19, 5);
    let (window_size_x, window_size_y) = (3, 4);
    let (array_size_x, array_size_y) = (3, 4);
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let rb = dataset.rasterband(band_index).unwrap();
    let values = rb
        .read_as_array::<u8>(
            (left, top),
            (window_size_x, window_size_y),
            (array_size_x, array_size_y),
        )
        .unwrap();

    let data = arr2(&[
        [226, 225, 157],
        [215, 222, 225],
        [213, 231, 229],
        [171, 189, 192],
    ]);

    assert_eq!(values, data);
    assert_eq!(rb.band_type(), GDALDataType::GDT_Byte);
}

#[test]
#[cfg(feature = "ndarray")]
fn test_read_block_as_array() {
    let band_index = 1;
    let block_index = (0, 0);
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let rasterband = dataset.rasterband(band_index).unwrap();
    let result = rasterband.read_block::<u8>(block_index);
    assert!(result.is_ok());
}

#[test]
#[cfg(feature = "ndarray")]
fn test_read_block_dimension() {
    let band_index = 1;
    let block = (0, 0);
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let rasterband = dataset.rasterband(band_index).unwrap();
    let array = rasterband.read_block::<u8>(block).unwrap();
    let dimension = (1, 100);
    assert_eq!(array.dim(), dimension);
}

#[test]
#[cfg(feature = "ndarray")]
fn test_read_block_last_dimension() {
    let band_index = 1;
    let block = (0, 49);
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let rasterband = dataset.rasterband(band_index).unwrap();
    let array = rasterband.read_block::<u8>(block).unwrap();
    let dimension = (1, 100);
    assert_eq!(array.dim(), dimension);
}

#[test]
#[cfg(feature = "ndarray")]
fn test_read_block_data() {
    let band_index = 1;
    let block = (0, 0);
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let rasterband = dataset.rasterband(band_index).unwrap();
    let array = rasterband.read_block::<u8>(block).unwrap();
    assert_eq!(array[[0, 0]], 0);
    assert_eq!(array[[0, 1]], 9);
    assert_eq!(array[[0, 98]], 24);
    assert_eq!(array[[0, 99]], 51);
}

#[test]
fn test_get_band_type() {
    let driver = Driver::get("MEM").unwrap();
    let dataset = driver.create("", 20, 10, 1).unwrap();
    let rb = dataset.rasterband(1).unwrap();
    assert_eq!(rb.band_type(), GDALDataType::GDT_Byte);
}

#[test]
fn test_get_rasterband() {
    let driver = Driver::get("MEM").unwrap();
    let dataset = driver.create("", 20, 10, 1).unwrap();
    let rasterband = dataset.rasterband(1);
    assert!(rasterband.is_ok());
    let rasterband2 = dataset.rasterband(2);
    assert!(rasterband2.is_err());
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
#[allow(clippy::float_cmp)]
fn test_set_no_data_value() {
    let driver = Driver::get("MEM").unwrap();
    let dataset = driver.create("", 20, 10, 1).unwrap();
    let rasterband = dataset.rasterband(1).unwrap();
    assert_eq!(rasterband.no_data_value(), None);
    assert!(rasterband.set_no_data_value(1.23).is_ok());
    assert_eq!(rasterband.no_data_value(), Some(1.23));
}

#[test]
fn test_get_scale() {
    let dataset = Dataset::open(fixture!("offset_scaled_tinymarble.tif")).unwrap();
    let rasterband = dataset.rasterband(1).unwrap();
    let scale = rasterband.scale();
    assert_eq!(scale, Some(1.2));
}

#[test]
fn test_get_offset() {
    let dataset = Dataset::open(fixture!("offset_scaled_tinymarble.tif")).unwrap();
    let rasterband = dataset.rasterband(1).unwrap();
    let offset = rasterband.offset();
    assert_eq!(offset, Some(12.0));
}

#[test]
fn test_get_default_scale() {
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let rasterband = dataset.rasterband(1).unwrap();
    let scale = rasterband.scale();

    if cfg!(all(major_ge_3, minor_ge_1)) {
        // This behavior changed in 3.1.0
        // Since the default value is indistinguishable from "not set", None is returned. Unclear
        // if this is a bug or intended behavior, but tracked at:
        // https://github.com/OSGeo/gdal/issues/2579
        assert_eq!(scale, None);
    } else {
        // on gdal 2.x and gdal 3.0
        assert_eq!(scale, Some(1.0));
    }
}

#[test]
fn test_get_default_offset() {
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let rasterband = dataset.rasterband(1).unwrap();
    let offset = rasterband.offset();
    if cfg!(all(major_ge_3, minor_ge_1)) {
        // This behavior changed in 3.1.0
        // Since the default value is indistinguishable from "not set", None is returned.  Unclear
        // if this is a bug or intended behavior, but tracked at:
        // https://github.com/OSGeo/gdal/issues/2579
        assert_eq!(offset, None);
    } else {
        // on gdal 2.x and gdal 3.0
        assert_eq!(offset, Some(0.0));
    }
}

#[test]
fn test_get_rasterband_size() {
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let rasterband = dataset.rasterband(1).unwrap();
    let size = rasterband.size();
    assert_eq!(size, (100, 50));
}

#[test]
fn test_get_rasterband_block_size() {
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let rasterband = dataset.rasterband(1).unwrap();
    let size = rasterband.block_size();
    assert_eq!(size, (100, 1));
}

#[test]
#[cfg(any(all(major_ge_2, minor_ge_2), major_ge_3))] // GDAL 2.2 .. 2.x or >= 3
fn test_get_rasterband_actual_block_size() {
    let dataset = Dataset::open(fixture!("tinymarble.png")).unwrap();
    let rasterband = dataset.rasterband(1).unwrap();
    let size = rasterband.actual_block_size((0, 40));
    assert_eq!(size.unwrap(), (100, 1));
}

#[test]
fn test_read_overviews() {
    let dataset = Dataset::open(fixture!("tinymarble.tif")).unwrap();
    let rasterband = dataset.rasterband(1).unwrap();
    let overview_count = rasterband.overview_count().unwrap();
    assert_eq!(overview_count, 2);

    let overview_2 = rasterband.overview(0).unwrap();
    let overview_4 = rasterband.overview(1).unwrap();
    assert_eq!(rasterband.size(), (100, 50));
    assert_eq!(overview_2.size(), (50, 25));
    assert_eq!(overview_4.size(), (25, 13));
}

#[test]
fn test_fail_read_overviews() {
    let dataset = Dataset::open(fixture!("offset_scaled_tinymarble.tif")).unwrap();
    let rasterband = dataset.rasterband(1).unwrap();
    let overview_count = rasterband.overview_count().unwrap();
    assert_eq!(overview_count, 0);

    let overview_2 = rasterband.overview(0);
    assert!(overview_2.is_err());
}

#[test]
fn test_rasterband_lifetime() {
    let dataset: Dataset = Dataset::open(fixture!("tinymarble.tif")).unwrap();
    let rasterband = dataset.rasterband(1).unwrap();
    let overview = rasterband.overview(0).unwrap();

    assert!(rasterband.no_data_value().is_none());
    assert!(overview.no_data_value().is_none());

    drop(rasterband);
    assert!(overview.no_data_value().is_none());
}
