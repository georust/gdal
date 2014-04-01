use std::slice;
use std::libc::{c_int, c_char, c_double};
use std::str::raw;
use std::os::getenv;
use std::path::Path;
use super::driver::Driver;
use super::register_drivers;
use super::geom::Point;


#[link(name="gdal")]
extern {
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
    fn GDALSetGeoTransform(hDS: *(), padfTransform: *c_double) -> c_int;
    fn GDALGetGeoTransform(hDS: *(), padfTransform: *mut c_double) -> c_int;
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

pub struct Dataset {
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

    pub fn set_geo_transform(&self, tr: (f64, f64, f64, f64, f64, f64)) {
        let (tr_0, tr_1, tr_2, tr_3, tr_4, tr_5) = tr;
        let tr_vec: ~[c_double] = ~[tr_0, tr_1, tr_2, tr_3, tr_4, tr_5];

        let rv = unsafe {
            GDALSetGeoTransform(self.c_dataset, tr_vec.as_ptr())
        } as int;
        assert!(rv == 0);
    }

    pub fn get_geo_transform(&self) -> (f64, f64, f64, f64, f64, f64) {
        let mut tr: ~[c_double] = slice::with_capacity(6);
        for _ in range(0, 6) { tr.push(0.0); }
        let rv = unsafe {
            GDALGetGeoTransform(
                self.c_dataset,
                tr.as_mut_ptr()
            )
        } as int;
        assert!(rv == 0);
        return (tr[0], tr[1], tr[2], tr[3], tr[4], tr[5]);
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
        window: Point<int>,
        window_size: Point<uint>,
        buffer_size: Point<uint>
    ) -> ByteBuffer {
        let buffer_size_bytes = buffer_size.x * buffer_size.y;
        let mut data: ~[u8] = slice::with_capacity(buffer_size_bytes);
        for _ in range(0, buffer_size_bytes) { data.push(0u8); } // TODO zero fill
        unsafe {
            let c_band = GDALGetRasterBand(self.c_dataset, band_index as c_int);
            let rv = GDALRasterIO(
                c_band,
                GF_Read,
                window.x as c_int,
                window.y as c_int,
                window_size.x as c_int,
                window_size.y as c_int,
                data.as_mut_ptr() as *(),
                buffer_size.x as c_int,
                buffer_size.y as c_int,
                GDT_Byte,
                0,
                0
            ) as int;
            assert!(rv == 0);
        };
        return ByteBuffer{
            size: buffer_size,
            data: data,
        };
    }

    pub fn write_raster(
        &self,
        band_index: int,
        window: Point<int>,
        window_size: Point<uint>,
        buffer: ByteBuffer
    ) {
        assert_eq!(buffer.data.len(), buffer.size.x * buffer.size.y);
        unsafe {
            let c_band = GDALGetRasterBand(self.c_dataset, band_index as c_int);
            let rv = GDALRasterIO(
                c_band,
                GF_Write,
                window.x as c_int,
                window.y as c_int,
                window_size.x as c_int,
                window_size.y as c_int,
                buffer.data.as_ptr() as *(),
                buffer.size.x as c_int,
                buffer.size.y as c_int,
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


struct ByteBuffer {
    size: Point<uint>,
    data: ~[u8],
}


fn fixture_path(name: &str) -> Path {
    let envvar = "RUST_GDAL_TEST_FIXTURES";
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
    let rv = dataset.read_raster(1, Point(20, 30), Point(10, 10), Point(3, 5));
    assert_eq!(rv.size.x, 3);
    assert_eq!(rv.size.y, 5);
    assert_eq!(rv.data, ~[13, 3, 18, 6, 9, 1, 2, 9, 4, 6, 11, 4, 6, 2, 9]);
}


#[test]
fn test_write_raster() {
    use super::driver::get_driver;
    let driver = get_driver("MEM").unwrap();
    let dataset = driver.create("", 20, 10, 1).unwrap();

    // create a 2x1 raster
    let raster = ByteBuffer{size: Point(2, 1), data: ~[50u8, 20u8]};

    // epand it to fill the image (20x10)
    dataset.write_raster(1, Point(0, 0), Point(20, 10), raster);

    // read a pixel from the left side
    let left = dataset.read_raster(1, Point(5, 5), Point(1, 1), Point(1, 1));
    assert_eq!(left.data[0], 50u8);

    // read a pixel from the right side
    let right = dataset.read_raster(1, Point(15, 5), Point(1, 1), Point(1, 1));
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
fn test_create() {
    use super::driver::get_driver;
    let driver = get_driver("MEM").unwrap();
    let dataset = driver.create("", 10, 20, 3).unwrap();
    assert_eq!(dataset.get_raster_size(), (10, 20));
    assert_eq!(dataset.get_raster_count(), 3);
    assert_eq!(dataset.get_driver().get_short_name(), ~"MEM");
}


#[test]
fn test_create_copy() {
    use super::driver::get_driver;
    let driver = get_driver("MEM").unwrap();
    let dataset = open(&fixture_path("tinymarble.jpeg")).unwrap();
    let copy = dataset.create_copy(driver, "").unwrap();
    assert_eq!(copy.get_raster_size(), (100, 50));
    assert_eq!(copy.get_raster_count(), 3);
}


#[test]
fn test_geo_transform() {
    use super::driver::get_driver;
    let driver = get_driver("MEM").unwrap();
    let dataset = driver.create("", 20, 10, 1).unwrap();
    let transform = (0., 1., 0., 0., 0., 1.);
    dataset.set_geo_transform(transform);
    assert_eq!(dataset.get_geo_transform(), transform);
}
