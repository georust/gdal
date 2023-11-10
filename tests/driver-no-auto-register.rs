use gdal::{Dataset, DriverManager};
use std::path::{Path, PathBuf};

/// Returns the fully qualified path to `filename` in `${CARGO_MANIFEST_DIR}/fixtures`.
fn fixture(filename: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(filename)
}

#[test]
/// Sequentially run tests
fn test_driver_no_auto_register() {
    test_driver_manager_destruction();
    test_deregister_all_but_one();
    test_manually_registering_drivers();
}

fn test_manually_registering_drivers() {
    DriverManager::prevent_auto_registration();
    DriverManager::destroy();

    assert_eq!(DriverManager::count(), 0);

    assert!(Dataset::open(fixture("tinymarble.tif")).is_err());

    DriverManager::register_all();

    assert!(Dataset::open(fixture("tinymarble.tif")).is_ok());

    let driver = DriverManager::get_driver_by_name("GTiff").unwrap();

    DriverManager::deregister_driver(&driver);

    assert!(Dataset::open(fixture("tinymarble.tif")).is_err());

    DriverManager::register_driver(&driver);

    assert!(Dataset::open(fixture("tinymarble.tif")).is_ok());
}

fn test_deregister_all_but_one() {
    DriverManager::prevent_auto_registration();
    DriverManager::register_all();

    assert!(DriverManager::count() > 0);

    let mut driver_index = 0;
    for _ in 0..DriverManager::count() {
        let driver = DriverManager::get_driver(driver_index).unwrap();

        if driver.short_name() == "GTiff" {
            driver_index += 1;
            continue;
        }

        DriverManager::deregister_driver(&driver);
    }

    assert_eq!(DriverManager::count(), 1);
}

fn test_driver_manager_destruction() {
    DriverManager::prevent_auto_registration();

    assert_eq!(DriverManager::count(), 0);

    DriverManager::register_all();

    assert!(DriverManager::count() > 0);

    DriverManager::destroy();

    assert_eq!(DriverManager::count(), 0);

    DriverManager::register_all();

    assert!(DriverManager::count() > 0);
}
