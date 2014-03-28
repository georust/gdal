use std::slice;
use std::str::raw;
use std::os::getenv;
use std::path::Path;
use std::libc::{c_int, c_char};
use sync::mutex::{StaticMutex, MUTEX_INIT};


#[link(name = "gdal")]
extern {
    fn GDALVersionInfo(key: *c_char) -> *c_char;
    fn GDALCreate(
            hDriver: *(),
            pszFilename: *c_char,
            nXSize: c_int,
            nYSize: c_int,
            nBands: c_int,
            eBandType: c_int,
            papszOptions: **c_char
        ) -> *();
    fn GDALCreateCopy(
            hDriver: *(),
            pszFilename: *c_char,
            hSrcDS: *(),
            bStrict: c_int,
            papszOptions: **c_char,
            pfnProgres: *(),
            pProgressData: *()
        ) -> *();
    fn GDALOpen(pszFilename: *c_char, eAccess: c_int) -> *();
    fn GDALClose(hDS: *());
    fn GDALGetDatasetDriver(hDataset: *()) -> *();
    fn GDALGetRasterXSize(hDataset: *()) -> c_int;
    fn GDALGetRasterYSize(hDataset: *()) -> c_int;
    fn GDALGetRasterCount(hDataset: *()) -> c_int;
    fn GDALGetProjectionRef(hDS: *()) -> *c_char;
    fn GDALSetProjection(hDS: *(), pszProjection: *c_char) -> c_int;
    fn GDALGetRasterBand(hDS: *(), nBandId: c_int) -> *();
    fn GDALRasterIO(
            hBand: *(),
            eRWFlag: c_int,
            nXOff: c_int,
            nYOff: c_int,
            nXSize: c_int,
            nYSize: c_int,
            pData: *(),
            nBufXSize: c_int,
            nBufYSize: c_int,
            GDALDataType: c_int,
            nPixelSpace: c_int,
            nLineSpace: c_int
        ) -> c_int;
    fn GDALAllRegister();
    fn GDALGetDriverByName(pszName: *c_char) -> *();
    fn GDALGetDriverShortName(hDriver: *()) -> *c_char;
    fn GDALGetDriverLongName(hDriver: *()) -> *c_char;
}

static GA_ReadOnly:  c_int = 0;
static GA_Update:    c_int = 1;

static GDT_Unknown:  c_int = 0;
static GDT_Byte:     c_int = 1;
static GDT_UInt16:   c_int = 2;
static GDT_Int16:    c_int = 3;
static GDT_UInt32:   c_int = 4;
static GDT_Int32:    c_int = 5;
static GDT_Float32:  c_int = 6;
static GDT_Float64:  c_int = 7;
static GDT_CInt16:   c_int = 8;
static GDT_CInt32:   c_int = 9;
static GDT_CFloat32: c_int = 10;
static GDT_CFloat64: c_int = 11;

static GF_Read:      c_int = 0;
static GF_Write:     c_int = 1;


static mut LOCK: StaticMutex = MUTEX_INIT;
static mut registered_drivers: bool = false;

fn register_drivers() {
    unsafe {
        let _g = LOCK.lock();
        if ! registered_drivers {
            GDALAllRegister();
            registered_drivers = true;
        }
    }
}


pub fn version_info(key: &str) -> ~str {
    let info = key.with_c_str(|c_key| {
        unsafe {
            let rv = GDALVersionInfo(c_key);
            return raw::from_c_str(rv);
        };
    });
    return info;
}


struct Dataset {
    c_dataset: *(),
}


impl Drop for Dataset {
    fn drop(&mut self) {
        unsafe { GDALClose(self.c_dataset); }
    }
}


impl Dataset {
    pub fn get_raster_size(&self) -> (int, int) {
        let size_x = unsafe { GDALGetRasterXSize(self.c_dataset) } as int;
        let size_y = unsafe { GDALGetRasterYSize(self.c_dataset) } as int;
        return (size_x, size_y);
    }

    pub fn get_driver(&self) -> Driver {
        let c_driver = unsafe { GDALGetDatasetDriver(self.c_dataset) };
        return Driver{c_driver: c_driver};
    }

    pub fn get_raster_count(&self) -> int {
        return unsafe { GDALGetRasterCount(self.c_dataset) } as int;
    }

    pub fn get_projection(&self) -> ~str {
        unsafe {
            let rv = GDALGetProjectionRef(self.c_dataset);
            return raw::from_c_str(rv);
        }
    }

    pub fn set_projection(&self, projection: &str) {
        projection.with_c_str(|c_projection| {
            unsafe { GDALSetProjection(self.c_dataset, c_projection) };
        });
    }

    pub fn create_copy(
        &self,
        driver: Driver,
        filename: &str
    ) -> Option<Dataset> {
        use std::ptr::null;
        let c_dataset = filename.with_c_str(|c_filename| {
            unsafe {
                return GDALCreateCopy(
                    driver.c_driver,
                    c_filename,
                    self.c_dataset,
                    0,
                    null(),
                    null(),
                    null()
                )
            }
        });
        return match c_dataset.is_null() {
            true  => None,
            false => Some(Dataset{c_dataset: c_dataset}),
        };
    }

    pub fn read_raster(
        &self,
        band_index: int,
        window_x: int, window_y: int,
        window_width: uint, window_height: uint,
        buffer_width: uint, buffer_height: uint
    ) -> ByteBuffer {
        let buffer_size = buffer_width * buffer_height;
        let mut data: ~[u8] = slice::with_capacity(buffer_size);
        for _ in range(0, buffer_size) { data.push(0u8); } // TODO zero fill
        unsafe {
            let c_band = GDALGetRasterBand(self.c_dataset, band_index as c_int);
            let rv = GDALRasterIO(
                c_band,
                GF_Read,
                window_x as c_int,
                window_y as c_int,
                window_width as c_int,
                window_height as c_int,
                data.as_mut_ptr() as *(),
                buffer_width as c_int,
                buffer_height as c_int,
                GDT_Byte,
                0,
                0
            ) as int;
            assert!(rv == 0);
        };
        return ByteBuffer{
            width: buffer_width,
            height: buffer_height,
            data: data,
        };
    }

    pub fn write_raster(
        &self,
        band_index: int,
        window_x: int, window_y: int,
        window_width: uint, window_height: uint,
        buffer: ByteBuffer
    ) {
        unsafe {
            let c_band = GDALGetRasterBand(self.c_dataset, band_index as c_int);
            let rv = GDALRasterIO(
                c_band,
                GF_Write,
                window_x as c_int,
                window_y as c_int,
                window_width as c_int,
                window_height as c_int,
                buffer.data.as_ptr() as *(),
                buffer.width as c_int,
                buffer.height as c_int,
                GDT_Byte,
                0,
                0
            ) as int;
            assert!(rv == 0);
        };
    }
}


pub fn open(path: &Path) -> Option<Dataset> {
    register_drivers();
    let filename = path.as_str().unwrap();
    let c_dataset = filename.with_c_str(|c_filename| {
        return unsafe { GDALOpen(c_filename, GA_ReadOnly) };
    });
    return match c_dataset.is_null() {
        true  => None,
        false => Some(Dataset{c_dataset: c_dataset}),
    };
}


struct Driver {
    c_driver: *(),
}


impl Driver {
    pub fn get_short_name(&self) -> ~str {
        unsafe {
            let rv = GDALGetDriverShortName(self.c_driver);
            return raw::from_c_str(rv);
        }
    }

    pub fn get_long_name(&self) -> ~str {
        unsafe {
            let rv = GDALGetDriverLongName(self.c_driver);
            return raw::from_c_str(rv);
        }
    }

    pub fn create(
        &self,
        filename: &str,
        size_x: int,
        size_y: int,
        bands: int
    ) -> Option<Dataset> {
        use std::ptr::null;
        let c_dataset = filename.with_c_str(|c_filename| {
            unsafe {
                return GDALCreate(
                    self.c_driver,
                    c_filename,
                    size_x as c_int,
                    size_y as c_int,
                    bands as c_int,
                    GDT_Byte,
                    null()
                );
            }
        });
        return match c_dataset.is_null() {
            true  => None,
            false => Some(Dataset{c_dataset: c_dataset}),
        };
    }
}


pub fn get_driver(name: &str) -> Option<Driver> {
    register_drivers();
    let c_driver = name.with_c_str(|c_name| {
        return unsafe { GDALGetDriverByName(c_name) };
    });
    return match c_driver.is_null() {
        true  => None,
        false => Some(Driver{c_driver: c_driver}),
    };
}


struct ByteBuffer {
    width: uint,
    height: uint,
    data: ~[u8],
}


#[test]
fn test_version_info() {
    let release_date = version_info("RELEASE_DATE");
    let release_name = version_info("RELEASE_NAME");
    let version_text = version_info("--version");

    let expected_text: ~str = "GDAL " + release_name + ", " +
        "released " + release_date.slice(0, 4) + "/" +
        release_date.slice(4, 6) + "/" + release_date.slice(6, 8);

    assert_eq!(version_text.into_owned(), expected_text);
}


fn fixture_path(name: &str) -> Path {
    let envvar = "RUSTILES_TEST_FIXTURES";
    let fixtures = match getenv(envvar) {
        Some(p) => Path::new(p),
        None => fail!("Environment variable {} not set", envvar)
    };
    let rv = fixtures.join(name);
    return rv;
}


#[test]
fn test_open() {
    let dataset = open(&fixture_path("tinymarble.jpeg"));
    assert!(dataset.is_some());

    let missing_dataset = open(&fixture_path("no_such_file.jpeg"));
    assert!(missing_dataset.is_none());
}


#[test]
fn test_get_raster_size() {
    let dataset = open(&fixture_path("tinymarble.jpeg")).unwrap();
    let (size_x, size_y) = dataset.get_raster_size();
    assert_eq!(size_x, 100);
    assert_eq!(size_y, 50);
}


#[test]
fn test_get_raster_count() {
    let dataset = open(&fixture_path("tinymarble.jpeg")).unwrap();
    let count = dataset.get_raster_count();
    assert_eq!(count, 3);
}


#[test]
fn test_get_projection() {
    let dataset = open(&fixture_path("tinymarble.jpeg")).unwrap();
    //dataset.set_projection("WGS84");
    let projection = dataset.get_projection();
    assert_eq!(projection.slice(0, 16), "GEOGCS[\"WGS 84\",");
}


#[test]
fn test_read_raster() {
    let dataset = open(&fixture_path("tinymarble.jpeg")).unwrap();
    let raster = dataset.read_raster(1, 20, 30, 10, 10, 3, 5);
    assert_eq!(raster.width, 3);
    assert_eq!(raster.height, 5);
    assert_eq!(raster.data, ~[13, 3, 18, 6, 9, 1, 2, 9, 4, 6, 11, 4, 6, 2, 9]);
}


#[test]
fn test_write_raster() {
    let driver = get_driver("MEM").unwrap();
    let dataset = driver.create("", 20, 10, 1).unwrap();

    // create a 2x1 raster
    let raster = ByteBuffer{width: 2, height: 1, data: ~[50u8, 20u8]};

    // epand it to fill the image (20x10)
    dataset.write_raster(1, 0, 0, 20, 10, raster);

    // read a pixel from the left side
    let left = dataset.read_raster(1, 5, 5, 1, 1, 1, 1);
    assert_eq!(left.data[0], 50u8);

    // read a pixel from the right side
    let right = dataset.read_raster(1, 15, 5, 1, 1, 1, 1);
    assert_eq!(right.data[0], 20u8);
}


#[test]
fn test_get_dataset_driver() {
    let dataset = open(&fixture_path("tinymarble.jpeg")).unwrap();
    let driver = dataset.get_driver();
    assert_eq!(driver.get_short_name(), ~"JPEG");
    assert_eq!(driver.get_long_name(), ~"JPEG JFIF");
}


#[test]
fn test_get_driver_by_name() {
    let missing_driver = get_driver("wtf");
    assert!(missing_driver.is_none());

    let ok_driver = get_driver("GTiff");
    assert!(ok_driver.is_some());
    let driver = ok_driver.unwrap();
    assert_eq!(driver.get_short_name(), ~"GTiff");
    assert_eq!(driver.get_long_name(), ~"GeoTIFF");
}


#[test]
fn test_create() {
    let driver = get_driver("MEM").unwrap();
    let dataset = driver.create("", 10, 20, 3).unwrap();
    assert_eq!(dataset.get_raster_size(), (10, 20));
    assert_eq!(dataset.get_raster_count(), 3);
    assert_eq!(dataset.get_driver().get_short_name(), ~"MEM");
}


#[test]
fn test_create_copy() {
    let driver = get_driver("MEM").unwrap();
    let dataset = open(&fixture_path("tinymarble.jpeg")).unwrap();
    let copy = dataset.create_copy(driver, "").unwrap();
    assert_eq!(copy.get_raster_size(), (100, 50));
    assert_eq!(copy.get_raster_count(), 3);
}
