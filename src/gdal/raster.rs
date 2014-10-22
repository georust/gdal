use libc::{c_int, c_char, c_double};
use super::geom::Point;
use sync::mutex::{StaticMutex, MUTEX_INIT};
use utils::_string;


#[link(name="gdal")]
extern {
    fn GDALAllRegister();
    fn GDALGetDriverByName(pszName: *const c_char) -> *const ();
    fn GDALGetDriverShortName(hDriver: *const ()) -> *const c_char;
    fn GDALGetDriverLongName(hDriver: *const ()) -> *const c_char;
    fn GDALCreate(
            hDriver: *const (),
            pszFilename: *const c_char,
            nXSize: c_int,
            nYSize: c_int,
            nBands: c_int,
            eBandType: c_int,
            papszOptions: *const *const c_char
        ) -> *const ();
    fn GDALCreateCopy(
            hDriver: *const (),
            pszFilename: *const c_char,
            hSrcDS: *const (),
            bStrict: c_int,
            papszOptions: *const *const c_char,
            pfnProgres: *const (),
            pProgressData: *const ()
        ) -> *const ();
    fn GDALOpen(pszFilename: *const c_char, eAccess: c_int) -> *const ();
    fn GDALClose(hDS: *const ());
    fn GDALGetDatasetDriver(hDataset: *const ()) -> *const ();
    fn GDALGetRasterXSize(hDataset: *const ()) -> c_int;
    fn GDALGetRasterYSize(hDataset: *const ()) -> c_int;
    fn GDALGetRasterCount(hDataset: *const ()) -> c_int;
    fn GDALGetProjectionRef(hDS: *const ()) -> *const c_char;
    fn GDALSetProjection(hDS: *const (), pszProjection: *const c_char) -> c_int;
    fn GDALSetGeoTransform(hDS: *const (), padfTransform: *const c_double) -> c_int;
    fn GDALGetGeoTransform(hDS: *const (), padfTransform: *mut c_double) -> c_int;
    fn GDALGetRasterBand(hDS: *const (), nBandId: c_int) -> *const ();
    fn GDALRasterIO(
            hBand: *const (),
            eRWFlag: c_int,
            nXOff: c_int,
            nYOff: c_int,
            nXSize: c_int,
            nYSize: c_int,
            pData: *const (),
            nBufXSize: c_int,
            nBufYSize: c_int,
            GDALDataType: c_int,
            nPixelSpace: c_int,
            nLineSpace: c_int
        ) -> c_int;
}

static GA_READONLY:  c_int = 0;
static GA_UPDATE:    c_int = 1;

static GDT_UNKNOWN:  c_int = 0;
static GDT_BYTE:     c_int = 1;
static GDT_UINT16:   c_int = 2;
static GDT_INT16:    c_int = 3;
static GDT_UINT32:   c_int = 4;
static GDT_INT32:    c_int = 5;
static GDT_FLOAT32:  c_int = 6;
static GDT_FLOAT64:  c_int = 7;
static GDT_CINT16:   c_int = 8;
static GDT_CINT32:   c_int = 9;
static GDT_CFLOAT32: c_int = 10;
static GDT_CFLOAT64: c_int = 11;

static GF_READ:      c_int = 0;
static GF_WRITE:     c_int = 1;

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


pub struct Driver {
    c_driver: *const (),
}


impl Driver {
    pub unsafe fn with_ptr(c_driver: *const ()) -> Driver {
        return Driver{c_driver: c_driver};
    }

    pub unsafe fn get_ptr(&self) -> *const () {
        return self.c_driver;
    }

    pub fn get_short_name(&self) -> String {
        let rv = unsafe { GDALGetDriverShortName(self.c_driver) };
        return _string(rv);
    }

    pub fn get_long_name(&self) -> String {
        let rv = unsafe { GDALGetDriverLongName(self.c_driver) };
        return _string(rv);
    }

    pub fn create(
        &self,
        filename: &str,
        size_x: int,
        size_y: int,
        bands: int
    ) -> Option<RasterDataset> {
        use std::ptr::null;
        let c_dataset = filename.with_c_str(|c_filename| {
            unsafe {
                return GDALCreate(
                    self.c_driver,
                    c_filename,
                    size_x as c_int,
                    size_y as c_int,
                    bands as c_int,
                    GDT_BYTE,
                    null()
                );
            }
        });
        return match c_dataset.is_null() {
            true  => None,
            false => unsafe { Some(RasterDataset::with_ptr(c_dataset)) },
        };
    }
}


pub struct RasterDataset {
    c_dataset: *const (),
}


impl Drop for RasterDataset {
    fn drop(&mut self) {
        unsafe { GDALClose(self.c_dataset); }
    }
}


impl RasterDataset {
    pub unsafe fn with_ptr(c_dataset: *const ()) -> RasterDataset {
        return RasterDataset{c_dataset: c_dataset};
    }

    pub unsafe fn get_ptr(&self) -> *const () {
        return self.c_dataset;
    }

    pub fn get_raster_size(&self) -> (int, int) {
        let size_x = unsafe { GDALGetRasterXSize(self.c_dataset) } as int;
        let size_y = unsafe { GDALGetRasterYSize(self.c_dataset) } as int;
        return (size_x, size_y);
    }

    pub fn get_driver(&self) -> Driver {
        unsafe {
            let c_driver = GDALGetDatasetDriver(self.c_dataset);
            return Driver::with_ptr(c_driver);
        };
    }

    pub fn get_raster_count(&self) -> int {
        return unsafe { GDALGetRasterCount(self.c_dataset) } as int;
    }

    pub fn get_projection(&self) -> String {
        let rv = unsafe { GDALGetProjectionRef(self.c_dataset) };
        return _string(rv);
    }

    pub fn set_projection(&self, projection: &str) {
        projection.with_c_str(|c_projection| {
            unsafe { GDALSetProjection(self.c_dataset, c_projection) };
        });
    }

    pub fn set_geo_transform(&self, tr: &[f64]) {
        assert_eq!(tr.len(), 6);
        let rv = unsafe {
            GDALSetGeoTransform(self.c_dataset, tr.as_ptr())
        } as int;
        assert!(rv == 0);
    }

    pub fn get_geo_transform(&self) -> Vec<f64> {
        let mut tr: Vec<c_double> = Vec::with_capacity(6);
        for _ in range(0i, 6) { tr.push(0.0); }
        let rv = unsafe {
            GDALGetGeoTransform(
                self.c_dataset,
                tr.as_mut_ptr()
            )
        } as int;
        assert!(rv == 0);
        return tr;
    }

    pub fn create_copy(
        &self,
        driver: Driver,
        filename: &str
    ) -> Option<RasterDataset> {
        use std::ptr::null;
        let c_dataset = filename.with_c_str(|c_filename| {
            unsafe {
                return GDALCreateCopy(
                    driver.get_ptr(),
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
            false => Some(RasterDataset{c_dataset: c_dataset}),
        };
    }

    pub fn read_raster(&self,
        band_index: int,
        window: Point<int>,
        window_size: Point<uint>,
        size: Point<uint>
        ) -> ByteBuffer
    {
        let nbytes = size.x * size.y;
        let mut data: Vec<u8> = range(0, nbytes).map(|_| 0u8).collect();
        unsafe {
            let c_band = GDALGetRasterBand(self.c_dataset, band_index as c_int);
            let rv = GDALRasterIO(
                c_band,
                GF_READ,
                window.x as c_int,
                window.y as c_int,
                window_size.x as c_int,
                window_size.y as c_int,
                data.as_mut_ptr() as *const (),
                size.x as c_int,
                size.y as c_int,
                GDT_BYTE,
                0,
                0
            ) as int;
            assert!(rv == 0);
        };
        return ByteBuffer{
            size: size,
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
                GF_WRITE,
                window.x as c_int,
                window.y as c_int,
                window_size.x as c_int,
                window_size.y as c_int,
                buffer.data.as_ptr() as *const (),
                buffer.size.x as c_int,
                buffer.size.y as c_int,
                GDT_BYTE,
                0,
                0
            ) as int;
            assert!(rv == 0);
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


pub fn open(path: &Path) -> Option<RasterDataset> {
    register_drivers();
    let filename = path.as_str().unwrap();
    let c_dataset = filename.with_c_str(|c_filename| {
        return unsafe { GDALOpen(c_filename, GA_READONLY) };
    });
    return match c_dataset.is_null() {
        true  => None,
        false => Some(RasterDataset{c_dataset: c_dataset}),
    };
}


pub struct ByteBuffer {
    size: Point<uint>,
    data: Vec<u8>,
}


#[cfg(test)]
mod test {
    use std::os::getenv;
    use std::path::Path;
    use super::super::geom::Point;
    use super::{ByteBuffer, get_driver, open};


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
        let dataset = open(&fixture_path("tinymarble.png"));
        assert!(dataset.is_some());

        let missing_dataset = open(&fixture_path("no_such_file.png"));
        assert!(missing_dataset.is_none());
    }


    #[test]
    fn test_get_raster_size() {
        let dataset = open(&fixture_path("tinymarble.png")).unwrap();
        let (size_x, size_y) = dataset.get_raster_size();
        assert_eq!(size_x, 100);
        assert_eq!(size_y, 50);
    }


    #[test]
    fn test_get_raster_count() {
        let dataset = open(&fixture_path("tinymarble.png")).unwrap();
        let count = dataset.get_raster_count();
        assert_eq!(count, 3);
    }


    #[test]
    fn test_get_projection() {
        let dataset = open(&fixture_path("tinymarble.png")).unwrap();
        //dataset.set_projection("WGS84");
        let projection = dataset.get_projection();
        assert_eq!(projection.as_slice().slice(0, 16), "GEOGCS[\"WGS 84\",");
    }


    #[test]
    fn test_read_raster() {
        let dataset = open(&fixture_path("tinymarble.png")).unwrap();
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
        let driver = get_driver("MEM").unwrap();
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
        let dataset = open(&fixture_path("tinymarble.png")).unwrap();
        let driver = dataset.get_driver();
        assert_eq!(driver.get_short_name().as_slice(), "PNG");
        assert_eq!(driver.get_long_name().as_slice(), "Portable Network Graphics");
    }


    #[test]
    fn test_create() {
        let driver = get_driver("MEM").unwrap();
        let dataset = driver.create("", 10, 20, 3).unwrap();
        assert_eq!(dataset.get_raster_size(), (10, 20));
        assert_eq!(dataset.get_raster_count(), 3);
        assert_eq!(dataset.get_driver().get_short_name().as_slice(), "MEM");
    }


    #[test]
    fn test_create_copy() {
        let driver = get_driver("MEM").unwrap();
        let dataset = open(&fixture_path("tinymarble.png")).unwrap();
        let copy = dataset.create_copy(driver, "").unwrap();
        assert_eq!(copy.get_raster_size(), (100, 50));
        assert_eq!(copy.get_raster_count(), 3);
    }


    #[test]
    fn test_geo_transform() {
        let driver = get_driver("MEM").unwrap();
        let dataset = driver.create("", 20, 10, 1).unwrap();
        let transform = vec!(0., 1., 0., 0., 0., 1.);
        dataset.set_geo_transform(transform.as_slice());
        assert_eq!(dataset.get_geo_transform(), transform);
    }


    #[test]
    fn test_get_driver_by_name() {
        let missing_driver = get_driver("wtf");
        assert!(missing_driver.is_none());

        let ok_driver = get_driver("GTiff");
        assert!(ok_driver.is_some());
        let driver = ok_driver.unwrap();
        assert_eq!(driver.get_short_name().as_slice(), "GTiff");
        assert_eq!(driver.get_long_name().as_slice(), "GeoTIFF");
    }
}
