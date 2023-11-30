use crate::dataset::Dataset;
use crate::metadata::Metadata;
use crate::raster::rasterband::ResampleAlg;
use crate::raster::{
    ByteBuffer, ColorEntry, ColorInterpretation, ColorTable, GdalDataType, RasterCreationOption,
    StatisticsAll, StatisticsMinMax,
};
use crate::test_utils::{fixture, TempFixture};
use crate::vsi::unlink_mem_file;
use crate::DriverManager;
use std::path::Path;
use std::str::FromStr;

#[cfg(feature = "ndarray")]
use ndarray::arr2;

#[test]
fn test_open() {
    let dataset = Dataset::open(fixture("tinymarble.tif"));
    assert!(dataset.is_ok());

    let missing_dataset = Dataset::open(fixture("no_such_file.png"));
    assert!(missing_dataset.is_err());
}

#[test]
fn test_get_raster_size() {
    let dataset = Dataset::open(fixture("tinymarble.tif")).unwrap();
    let (size_x, size_y) = dataset.raster_size();
    assert_eq!(size_x, 100);
    assert_eq!(size_y, 50);
}

#[test]
fn test_get_raster_count() {
    let dataset = Dataset::open(fixture("tinymarble.tif")).unwrap();
    let count = dataset.raster_count();
    assert_eq!(count, 3);
}

#[test]
fn test_get_projection() {
    let dataset = Dataset::open(fixture("tinymarble.tif")).unwrap();
    let projection = dataset.projection();
    assert_eq!(
        projection.chars().take(16).collect::<String>(),
        "GEOGCS[\"WGS 84\","
    );
}

#[test]
fn test_read_raster() {
    let dataset = Dataset::open(fixture("tinymarble.tif")).unwrap();
    let rb = dataset.rasterband(1).unwrap();
    let rv = rb.read_as::<u8>((20, 30), (2, 3), (2, 3), None).unwrap();
    assert_eq!(rv.size.0, 2);
    assert_eq!(rv.size.1, 3);
    assert_eq!(rv.data, vec!(7, 7, 7, 10, 8, 12));

    let mut buf = rv;
    rb.read_into_slice((20, 30), (2, 3), (2, 3), &mut buf.data, None)
        .unwrap();
    assert_eq!(buf.data, vec!(7, 7, 7, 10, 8, 12));
}

#[test]
fn test_read_raster_with_default_resample() {
    let dataset = Dataset::open(fixture("tinymarble.tif")).unwrap();
    let rb = dataset.rasterband(1).unwrap();
    let rv = rb.read_as::<u8>((20, 30), (4, 4), (2, 2), None).unwrap();
    assert_eq!(rv.size.0, 2);
    assert_eq!(rv.size.1, 2);
    assert_eq!(rv.data, vec!(8, 7, 8, 11));

    let mut buf = rv;
    rb.read_into_slice((20, 30), (4, 4), (2, 2), &mut buf.data, None)
        .unwrap();
    assert_eq!(buf.data, vec!(8, 7, 8, 11));
}

#[test]
fn test_read_raster_with_average_resample() {
    let dataset = Dataset::open(fixture("tinymarble.tif")).unwrap();
    let rb = dataset.rasterband(1).unwrap();
    let resample_alg = ResampleAlg::Average;
    let rv = rb
        .read_as::<u8>((20, 30), (4, 4), (2, 2), Some(resample_alg))
        .unwrap();
    assert_eq!(rv.size.0, 2);
    assert_eq!(rv.size.1, 2);
    assert_eq!(rv.data, vec!(8, 7, 8, 11));

    let mut buf = rv;
    rb.read_into_slice((20, 30), (4, 4), (2, 2), &mut buf.data, Some(resample_alg))
        .unwrap();
    assert_eq!(buf.data, vec!(8, 7, 8, 11));
}

#[test]
fn test_write_raster() {
    let driver = DriverManager::get_driver_by_name("MEM").unwrap();
    let dataset = driver.create("", 20, 10, 1).unwrap();

    // create a 2x1 raster
    let raster = ByteBuffer {
        size: (2, 1),
        data: vec![50u8, 20u8],
    };

    // epand it to fill the image (20x10)
    let mut rb = dataset.rasterband(1).unwrap();

    let res = rb.write((0, 0), (20, 10), &raster);

    assert!(res.is_ok());

    // read a pixel from the left side
    let left = rb.read_as::<u8>((5, 5), (1, 1), (1, 1), None).unwrap();
    assert_eq!(left.data[0], 50u8);

    // read a pixel from the right side
    let right = rb.read_as::<u8>((15, 5), (1, 1), (1, 1), None).unwrap();
    assert_eq!(right.data[0], 20u8);
}

#[test]
fn test_rename_remove_raster() {
    let dataset = Dataset::open(fixture("tinymarble.tif")).unwrap();

    let mem_file_path_a = Path::new("/vsimem/030bd1d1-8955-4604-8e37-177dade13863");
    let mem_file_path_b = Path::new("/vsimem/c7bfce32-2474-48fa-a907-2af95f83c824");

    let driver = DriverManager::get_driver_by_name("GTiff").unwrap();

    dataset.create_copy(&driver, mem_file_path_a, &[]).unwrap();

    driver.rename(mem_file_path_b, mem_file_path_a).unwrap();

    // old dataset path is gone
    assert!(Dataset::open(mem_file_path_a).is_err());
    // dataset exists under new name
    Dataset::open(mem_file_path_b).unwrap();

    driver.delete(mem_file_path_b).unwrap();

    assert!(Dataset::open(mem_file_path_b).is_err());
}

#[test]
fn test_create() {
    let driver = DriverManager::get_driver_by_name("MEM").unwrap();
    let dataset = driver.create("", 10, 20, 3).unwrap();
    assert_eq!(dataset.raster_size(), (10, 20));
    assert_eq!(dataset.raster_count(), 3);
    assert_eq!(dataset.driver().short_name(), "MEM");
}

#[test]
fn test_create_with_band_type() {
    let driver = DriverManager::get_driver_by_name("MEM").unwrap();
    let dataset = driver
        .create_with_band_type::<f32, _>("", 10, 20, 3)
        .unwrap();
    assert_eq!(dataset.raster_size(), (10, 20));
    assert_eq!(dataset.raster_count(), 3);
    assert_eq!(dataset.driver().short_name(), "MEM");
    let rb = dataset.rasterband(1).unwrap();
    assert_eq!(rb.band_type(), GdalDataType::Float32);
}

#[test]
fn test_create_with_band_type_with_options() {
    let driver = DriverManager::get_driver_by_name("GTiff").unwrap();
    let options = [
        RasterCreationOption {
            key: "TILED",
            value: "YES",
        },
        RasterCreationOption {
            key: "BLOCKXSIZE",
            value: "128",
        },
        RasterCreationOption {
            key: "BLOCKYSIZE",
            value: "64",
        },
        RasterCreationOption {
            key: "COMPRESS",
            value: "LZW",
        },
        RasterCreationOption {
            key: "INTERLEAVE",
            value: "BAND",
        },
    ];

    let tmp_filename = TempFixture::empty("test.tif");
    {
        let dataset = driver
            .create_with_band_type_with_options::<u8, _>(&tmp_filename, 256, 256, 1, &options)
            .unwrap();
        let rasterband = dataset.rasterband(1).unwrap();
        let block_size = rasterband.block_size();
        assert_eq!(block_size, (128, 64));
    }

    let dataset = Dataset::open(tmp_filename).unwrap();
    let key = "INTERLEAVE";
    let domain = "IMAGE_STRUCTURE";
    let meta = dataset.metadata_item(key, domain);
    assert_eq!(meta.as_deref(), Some("BAND"));
    let key = "COMPRESSION";
    let domain = "IMAGE_STRUCTURE";
    let meta = dataset.metadata_item(key, domain);
    assert_eq!(meta.as_deref(), Some("LZW"));
}

#[test]
fn test_create_copy() {
    let driver = DriverManager::get_driver_by_name("MEM").unwrap();
    let dataset = Dataset::open(fixture("tinymarble.tif")).unwrap();
    let copy = dataset.create_copy(&driver, "", &[]).unwrap();
    assert_eq!(copy.raster_size(), (100, 50));
    assert_eq!(copy.raster_count(), 3);
}

#[test]
fn test_create_copy_with_options() {
    let dataset = Dataset::open(fixture("tinymarble.tif")).unwrap();

    assert_eq!(
        dataset.metadata_domain("IMAGE_STRUCTURE").unwrap(),
        vec!["INTERLEAVE=PIXEL"]
    );

    let mem_file_path = "/vsimem/5128fad0-0a6b-4a9e-9899-ec78da7c6f04";

    let copy = dataset
        .create_copy(
            &DriverManager::get_driver_by_name("GTiff").unwrap(),
            mem_file_path,
            &[
                RasterCreationOption {
                    key: "INTERLEAVE",
                    value: "BAND",
                },
                RasterCreationOption {
                    key: "COMPRESS",
                    value: "LZW",
                },
            ],
        )
        .unwrap();

    assert_eq!(copy.raster_size(), (100, 50));
    assert_eq!(copy.raster_count(), 3);

    assert_eq!(
        copy.metadata_domain("IMAGE_STRUCTURE").unwrap(),
        vec!["COMPRESSION=LZW", "INTERLEAVE=BAND"]
    );

    unlink_mem_file(mem_file_path).unwrap();
}

#[test]
#[allow(clippy::float_cmp)]
fn test_geo_transform() {
    let driver = DriverManager::get_driver_by_name("MEM").unwrap();
    let mut dataset = driver.create("", 20, 10, 1).unwrap();
    let transform = [0., 1., 0., 0., 0., 1.];
    assert!(dataset.set_geo_transform(&transform).is_ok());
    assert_eq!(dataset.geo_transform().unwrap(), transform);
}

#[test]
fn test_get_driver_by_name() {
    let missing_driver = DriverManager::get_driver_by_name("wtf");
    assert!(missing_driver.is_err());

    let ok_driver = DriverManager::get_driver_by_name("GTiff");
    assert!(ok_driver.is_ok());
    let driver = ok_driver.unwrap();
    assert_eq!(driver.short_name(), "GTiff");
    assert_eq!(driver.long_name(), "GeoTIFF");
}

#[test]
fn test_read_raster_as() {
    let dataset = Dataset::open(fixture("tinymarble.tif")).unwrap();
    let rb = dataset.rasterband(1).unwrap();
    let rv = rb.read_as::<u8>((20, 30), (2, 3), (2, 3), None).unwrap();
    assert_eq!(rv.data, vec!(7, 7, 7, 10, 8, 12));
    assert_eq!(rv.size.0, 2);
    assert_eq!(rv.size.1, 3);
    assert_eq!(rb.band_type(), GdalDataType::UInt8);
}

#[test]
fn mask_flags() {
    let dataset = Dataset::open(fixture("tinymarble.tif")).unwrap();
    let rb = dataset.rasterband(1).unwrap();
    let mask_flags = rb.mask_flags().unwrap();
    assert!(!mask_flags.is_nodata());
    assert!(!mask_flags.is_alpha());
    assert!(!mask_flags.is_per_dataset());
    assert!(mask_flags.is_all_valid());
}

#[test]
fn open_mask_band() {
    let dataset = Dataset::open(fixture("tinymarble.tif")).unwrap();
    let rb = dataset.rasterband(1).unwrap();
    let mb = rb.open_mask_band().unwrap();
    let mask_values = mb.read_as::<u8>((20, 30), (2, 3), (2, 3), None).unwrap();
    assert_eq!(mask_values.data, [255u8; 6])
}

#[test]
fn create_mask_band() {
    let driver = DriverManager::get_driver_by_name("MEM").unwrap();
    let dataset = driver.create("", 20, 10, 1).unwrap();
    let mut rb = dataset.rasterband(1).unwrap();
    rb.create_mask_band(false).unwrap();

    let mb = rb.open_mask_band().unwrap();
    let mask_values = mb.read_as::<u8>((0, 0), (20, 10), (20, 10), None).unwrap();
    assert_eq!(mask_values.data, [0; 200])
}

#[test]
#[cfg(feature = "ndarray")]
fn test_read_raster_as_array() {
    let band_index = 1;
    let (left, top) = (19, 5);
    let (window_size_x, window_size_y) = (3, 4);
    let (array_size_x, array_size_y) = (3, 4);
    let dataset = Dataset::open(fixture("tinymarble.tif")).unwrap();
    let rb = dataset.rasterband(band_index).unwrap();
    let values = rb
        .read_as_array::<u8>(
            (left, top),
            (window_size_x, window_size_y),
            (array_size_x, array_size_y),
            None,
        )
        .unwrap();

    let data = arr2(&[
        [226, 225, 157],
        [215, 222, 225],
        [213, 231, 229],
        [171, 189, 192],
    ]);

    assert_eq!(values, data);
    assert_eq!(rb.band_type(), GdalDataType::UInt8);
}

#[test]
#[cfg(feature = "ndarray")]
fn test_read_block_as_array() {
    let band_index = 1;
    let block_index = (0, 0);
    let dataset = Dataset::open(fixture("tinymarble.tif")).unwrap();
    let rasterband = dataset.rasterband(band_index).unwrap();
    let result = rasterband.read_block::<u8>(block_index);
    assert!(result.is_ok());
}

#[test]
#[cfg(feature = "ndarray")]
fn test_read_block_dimension() {
    let band_index = 1;
    let block = (0, 0);
    let dataset = Dataset::open(fixture("tinymarble.tif")).unwrap();
    let rasterband = dataset.rasterband(band_index).unwrap();
    let array = rasterband.read_block::<u8>(block).unwrap();
    assert_eq!(array.dim(), (27, 100));
}

#[test]
#[cfg(feature = "ndarray")]
fn test_read_block_data() {
    let band_index = 1;
    let block = (0, 0);
    let dataset = Dataset::open(fixture("tinymarble.tif")).unwrap();
    let rasterband = dataset.rasterband(band_index).unwrap();
    let array = rasterband.read_block::<u8>(block).unwrap();
    assert_eq!(array[[0, 0]], 0);
    assert_eq!(array[[0, 1]], 9);
    assert_eq!(array[[0, 98]], 24);
    assert_eq!(array[[0, 99]], 51);
}

#[test]
fn test_get_band_type() {
    let driver = DriverManager::get_driver_by_name("MEM").unwrap();
    let dataset = driver.create("", 20, 10, 1).unwrap();
    let rb = dataset.rasterband(1).unwrap();
    assert_eq!(rb.band_type(), GdalDataType::UInt8);
}

#[test]
fn test_get_rasterband() {
    let driver = DriverManager::get_driver_by_name("MEM").unwrap();
    let dataset = driver.create("", 20, 10, 1).unwrap();
    let rasterband = dataset.rasterband(1);
    assert!(rasterband.is_ok());
    let rasterband2 = dataset.rasterband(2);
    assert!(rasterband2.is_err());
}

#[test]
fn test_get_no_data_value() {
    let dataset = Dataset::open(fixture("tinymarble.tif")).unwrap();
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
    let driver = DriverManager::get_driver_by_name("MEM").unwrap();
    let dataset = driver.create("", 20, 10, 1).unwrap();
    let mut rasterband = dataset.rasterband(1).unwrap();
    assert_eq!(rasterband.no_data_value(), None);
    assert!(rasterband.set_no_data_value(Some(1.23)).is_ok());
    assert_eq!(rasterband.no_data_value(), Some(1.23));
    assert!(rasterband.set_no_data_value(None).is_ok());
    assert_eq!(rasterband.no_data_value(), None);
}

#[test]
fn test_get_scale() {
    let dataset = Dataset::open(fixture("offset_scaled_tinymarble.tif")).unwrap();
    let rasterband = dataset.rasterband(1).unwrap();
    let scale = rasterband.scale();
    assert_eq!(scale, Some(1.2));
}

#[test]
fn test_get_offset() {
    let dataset = Dataset::open(fixture("offset_scaled_tinymarble.tif")).unwrap();
    let rasterband = dataset.rasterband(1).unwrap();
    let offset = rasterband.offset();
    assert_eq!(offset, Some(12.0));
}

#[test]
fn test_get_default_scale() {
    let dataset = Dataset::open(fixture("tinymarble.tif")).unwrap();
    let rasterband = dataset.rasterband(1).unwrap();
    let scale = rasterband.scale();

    // This is either `None` or `Some(1.0)`, see https://github.com/OSGeo/gdal/issues/2579
    assert_eq!(scale.unwrap_or(1.0), 1.0);
}

#[test]
fn test_get_default_offset() {
    let dataset = Dataset::open(fixture("tinymarble.tif")).unwrap();
    let rasterband = dataset.rasterband(1).unwrap();
    let offset = rasterband.offset();

    // This is either `None` or `Some(0.0)`, see https://github.com/OSGeo/gdal/issues/2579
    assert_eq!(offset.unwrap_or(0.0), 0.0);
}

#[test]
fn test_get_rasterband_size() {
    let dataset = Dataset::open(fixture("tinymarble.tif")).unwrap();
    let rasterband = dataset.rasterband(1).unwrap();
    let size = rasterband.size();
    assert_eq!(size, (100, 50));
}

#[test]
fn test_get_rasterband_block_size() {
    let dataset = Dataset::open(fixture("tinymarble.tif")).unwrap();
    let rasterband = dataset.rasterband(1).unwrap();
    let size = rasterband.block_size();
    assert_eq!(size, (100, 27));
}

#[test]
#[cfg(any(all(major_ge_2, minor_ge_2), major_ge_3))] // GDAL 2.2 .. 2.x or >= 3
fn test_get_rasterband_actual_block_size() {
    let dataset = Dataset::open(fixture("tinymarble.tif")).unwrap();
    let rasterband = dataset.rasterband(1).unwrap();
    let size = rasterband.actual_block_size(0, 0).unwrap();
    assert_eq!(size, (100, 27));
}

#[test]
fn test_read_overviews() {
    let dataset = Dataset::open(fixture("tinymarble.tif")).unwrap();
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
    let dataset = Dataset::open(fixture("offset_scaled_tinymarble.tif")).unwrap();
    let rasterband = dataset.rasterband(1).unwrap();
    let overview_count = rasterband.overview_count().unwrap();
    assert_eq!(overview_count, 0);

    let overview_2 = rasterband.overview(0);
    assert!(overview_2.is_err());
}

#[test]
fn test_rasterband_lifetime() {
    let dataset: Dataset = Dataset::open(fixture("tinymarble.tif")).unwrap();

    let overview = {
        let rasterband = dataset.rasterband(1).unwrap();
        let overview = rasterband.overview(0).unwrap();
        overview
    };

    assert!(overview.no_data_value().is_none());
}

#[test]
fn test_get_rasterband_color_interp() {
    let dataset = Dataset::open(fixture("tinymarble.tif")).unwrap();
    let rasterband = dataset.rasterband(1).unwrap();
    let band_interp = rasterband.color_interpretation();
    assert_eq!(band_interp, ColorInterpretation::RedBand);
}

#[test]
fn test_set_rasterband_color_interp() {
    let driver = DriverManager::get_driver_by_name("MEM").unwrap();
    let dataset = driver.create("", 1, 1, 1).unwrap();
    let mut rasterband = dataset.rasterband(1).unwrap();
    rasterband
        .set_color_interpretation(ColorInterpretation::AlphaBand)
        .unwrap();
    let band_interp = rasterband.color_interpretation();
    assert_eq!(band_interp, ColorInterpretation::AlphaBand);
}

#[test]
fn test_set_rasterband_scale() {
    let driver = DriverManager::get_driver_by_name("MEM").unwrap();
    let dataset = driver.create("", 1, 1, 1).unwrap();
    let mut rasterband = dataset.rasterband(1).unwrap();
    let scale = 1234.5678;
    rasterband.set_scale(scale).unwrap();
    assert_eq!(rasterband.scale().unwrap(), scale);
}

#[test]
fn test_set_rasterband_offset() {
    let driver = DriverManager::get_driver_by_name("MEM").unwrap();
    let dataset = driver.create("", 1, 1, 1).unwrap();
    let mut rasterband = dataset.rasterband(1).unwrap();
    let offset = -123.456;
    rasterband.set_offset(offset).unwrap();
    assert_eq!(rasterband.offset().unwrap(), offset);
}

#[test]
fn test_color_interp_names() {
    assert_eq!(ColorInterpretation::AlphaBand.name(), "Alpha");
    assert_eq!(
        ColorInterpretation::from_name("Alpha").unwrap(),
        ColorInterpretation::AlphaBand
    );
    assert_eq!(
        ColorInterpretation::from_name("not a valid name").unwrap(),
        ColorInterpretation::Undefined
    );
}

#[test]
fn test_rasterize() {
    let wkt = "POLYGON ((2 2, 2 4.25, 4.25 4.25, 4.25 2, 2 2))";
    let poly = crate::vector::Geometry::from_wkt(wkt).unwrap();

    let rows = 5;
    let cols = 5;
    let driver = DriverManager::get_driver_by_name("MEM").unwrap();
    let mut dataset = driver.create("", rows, cols, 1).unwrap();

    let bands = [1];
    let geometries = [poly];
    let burn_values = [1.0];
    super::rasterize(&mut dataset, &bands, &geometries, &burn_values, None).unwrap();

    let rb = dataset.rasterband(1).unwrap();
    let values = rb.read_as::<u8>((0, 0), (5, 5), (5, 5), None).unwrap();
    assert_eq!(
        values.data,
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0,]
    );
}

#[test]
fn test_rasterband_unit() {
    let dataset = Dataset::open(fixture("tinymarble.tif")).unwrap();
    let rasterband = dataset.rasterband(1).unwrap();

    assert!(rasterband.unit().is_empty());

    let dataset = Dataset::open(fixture("114p01_0100_deme_truncated.dem")).unwrap();
    let rasterband = dataset.rasterband(1).unwrap();

    assert_eq!(rasterband.unit(), "m".to_string());
}

#[test]
fn test_color_table() {
    use crate::raster::rasterband::{ColorEntry, PaletteInterpretation};

    // Raster containing one band.
    let dataset = Dataset::open(fixture("test_color_table.tif")).expect("open failure");
    assert_eq!(dataset.raster_count(), 1);

    // Band is PaletteIndex.
    let band = dataset.rasterband(1).expect("rasterband failure");
    assert_eq!(
        band.color_interpretation(),
        ColorInterpretation::PaletteIndex
    );

    // Color table is RGB.
    let color_table = band.color_table().unwrap();
    assert_eq!(
        color_table.palette_interpretation(),
        PaletteInterpretation::Rgba
    );

    // Color table has 256 entries.
    let entry_count = color_table.entry_count();
    assert_eq!(entry_count, 256);

    // Check that entry and entry_as_rgb are the same.
    for index in 0..entry_count {
        if let ColorEntry::Rgba(entry) = color_table.entry(index).unwrap() {
            let rgb_entry = color_table.entry_as_rgb(index).unwrap();
            assert_eq!(entry.r, rgb_entry.r);
            assert_eq!(entry.g, rgb_entry.g);
            assert_eq!(entry.b, rgb_entry.b);
            assert_eq!(entry.a, rgb_entry.a);
        } else {
            panic!();
        }
    }
}

#[test]
fn test_create_color_table() {
    let outfile = TempFixture::empty("color_labels.tif");
    // Open, modify, then close the base file.
    {
        let dataset = Dataset::open(fixture("labels.tif")).unwrap();
        // Confirm we have a band without a color table.
        assert_eq!(dataset.raster_count(), 1);
        let band = dataset.rasterband(1).unwrap();
        assert_eq!(band.band_type(), GdalDataType::UInt8);
        assert!(band.color_table().is_none());

        // Create a new file to put color table in
        let dataset = dataset
            .create_copy(&dataset.driver(), &outfile, &[])
            .unwrap();
        dataset
            .rasterband(1)
            .unwrap()
            .set_no_data_value(None)
            .unwrap();
        let mut ct = ColorTable::default();
        ct.set_color_entry(2, &ColorEntry::rgba(255, 0, 0, 255));
        ct.set_color_entry(5, &ColorEntry::rgba(0, 255, 0, 255));
        ct.set_color_entry(7, &ColorEntry::rgba(0, 0, 255, 255));

        assert_eq!(ct.entry_count(), 8);
        assert_eq!(ct.entry(0), Some(ColorEntry::rgba(0, 0, 0, 0)));
        assert_eq!(ct.entry(2), Some(ColorEntry::rgba(255, 0, 0, 255)));
        assert_eq!(ct.entry(8), None);

        dataset.rasterband(1).unwrap().set_color_table(&ct);
    }

    // Reopen to confirm the changes.
    let dataset = Dataset::open(&outfile).unwrap();
    let band = dataset.rasterband(1).unwrap();
    let ct = band.color_table().expect("saved color table");

    // Note: the GeoTIFF driver alters the palette, creating black entries to fill up all indexes
    // up to 255. Other drivers may do things differently.
    assert_eq!(ct.entry(0), Some(ColorEntry::rgba(0, 0, 0, 255)));
    assert_eq!(ct.entry(2), Some(ColorEntry::rgba(255, 0, 0, 255)));
    assert_eq!(ct.entry(5), Some(ColorEntry::rgba(0, 255, 0, 255)));
    assert_eq!(ct.entry(7), Some(ColorEntry::rgba(0, 0, 255, 255)));
    assert_eq!(ct.entry(8), Some(ColorEntry::rgba(0, 0, 0, 255)));
}

#[test]
fn test_color_ramp() {
    let ct = ColorTable::color_ramp(0, &ColorEntry::grey(0), 99, &ColorEntry::grey(99)).unwrap();
    assert_eq!(ct.entry(0), Some(ColorEntry::grey(0)));
    assert_eq!(ct.entry(57), Some(ColorEntry::grey(57)));
    assert_eq!(ct.entry(99), Some(ColorEntry::grey(99)));
    assert_eq!(ct.entry(100), None);
}

#[test]
fn test_raster_stats() {
    let fixture = TempFixture::fixture("tinymarble.tif");

    let dataset = Dataset::open(&fixture).unwrap();
    let rb = dataset.rasterband(1).unwrap();

    assert!(rb.get_statistics(false, false).unwrap().is_none());

    assert_eq!(
        rb.get_statistics(true, false).unwrap().unwrap(),
        StatisticsAll {
            min: 0.0,
            max: 255.0,
            mean: 68.4716,
            std_dev: 83.68444773934999,
        }
    );

    assert_eq!(
        rb.compute_raster_min_max(true).unwrap(),
        StatisticsMinMax {
            min: 0.0,
            max: 255.0,
        }
    );
}

#[test]
fn test_raster_histogram() {
    let fixture = TempFixture::fixture("tinymarble.tif");

    let dataset = Dataset::open(&fixture).unwrap();
    let rb = dataset.rasterband(1).unwrap();

    let hist = rb.default_histogram(false).unwrap();
    assert!(hist.is_none());

    let hist = rb.default_histogram(true).unwrap().unwrap();
    let expected = &[
        548, 104, 133, 127, 141, 125, 156, 129, 130, 117, 94, 94, 80, 81, 78, 63, 50, 66, 48, 48,
        33, 38, 41, 35, 41, 39, 32, 40, 26, 27, 25, 24, 18, 25, 29, 27, 20, 34, 17, 24, 29, 11, 20,
        21, 12, 19, 16, 16, 11, 10, 19, 5, 11, 10, 6, 9, 7, 12, 13, 6, 8, 7, 8, 14, 9, 14, 4, 8, 5,
        12, 6, 10, 7, 9, 8, 6, 3, 7, 5, 8, 9, 5, 4, 8, 3, 9, 3, 6, 11, 7, 6, 3, 9, 9, 7, 6, 9, 10,
        10, 4, 7, 2, 4, 7, 2, 12, 7, 10, 4, 6, 5, 2, 4, 5, 7, 3, 5, 7, 7, 14, 9, 12, 6, 6, 8, 5, 8,
        3, 3, 5, 11, 4, 9, 7, 14, 7, 10, 11, 6, 6, 5, 4, 9, 6, 6, 9, 5, 12, 11, 9, 3, 8, 5, 6, 4,
        2, 9, 7, 9, 9, 9, 6, 6, 8, 5, 9, 13, 4, 9, 4, 7, 13, 10, 5, 7, 8, 11, 12, 5, 17, 9, 11, 9,
        8, 9, 5, 8, 9, 5, 6, 9, 11, 8, 7, 7, 6, 7, 8, 8, 8, 5, 6, 7, 5, 8, 5, 6, 8, 7, 4, 8, 6, 5,
        11, 8, 8, 5, 4, 6, 4, 9, 7, 6, 6, 7, 7, 12, 6, 9, 17, 12, 20, 18, 17, 21, 24, 30, 29, 57,
        72, 83, 21, 11, 9, 18, 7, 13, 10, 2, 4, 0, 1, 3, 4, 1, 1,
    ];
    assert_eq!(hist.counts(), expected);

    let hist = rb.histogram(-0.5, 255.5, 128, true, true).unwrap();
    let expected_small = (0..expected.len())
        .step_by(2)
        .map(|i| expected[i] + expected[i + 1])
        .collect::<Vec<_>>();
    assert_eq!(hist.counts(), &expected_small);

    // n_buckets = 0 is not allowed
    let hist = rb.histogram(-0.5, 255.5, 0, true, true);
    hist.expect_err("histogram with 0 buckets should panic");
}

#[test]
fn test_resample_str() {
    assert!(ResampleAlg::from_str("foobar").is_err());

    for e in ResampleAlg::iter() {
        let stringed = e.to_string();
        let parsed = ResampleAlg::from_str(&stringed);
        assert!(parsed.is_ok(), "{stringed}");
        assert_eq!(parsed.unwrap(), e, "{stringed}");
    }
}
