use gdal::DriverManager;

#[test]
fn test_get_driver() {
    let driver = DriverManager::get_driver_by_name("GTiff").unwrap();
    assert_eq!(driver.short_name(), "GTiff");
    assert_eq!(driver.long_name(), "GeoTIFF");

    assert!(DriverManager::count() > 0);
    assert!(DriverManager::get_driver(0).is_ok());
}
