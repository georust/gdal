use libc::{c_int, c_char, c_double};
use std::ffi::CString;
use super::super::geom::Point;
use utils::_string;
use raster::{gdal, Driver};
use raster::driver::_register_drivers;


pub struct RasterDataset {
    c_dataset: *const (),
}


impl Drop for RasterDataset {
    fn drop(&mut self) {
        unsafe { gdal::GDALClose(self.c_dataset); }
    }
}


impl RasterDataset {
    pub unsafe fn _with_c_ptr(c_dataset: *const ()) -> RasterDataset {
        return RasterDataset{c_dataset: c_dataset};
    }

    pub unsafe fn _c_ptr(&self) -> *const () {
        return self.c_dataset;
    }

    pub fn size(&self) -> (isize, isize) {
        let size_x = unsafe { gdal::GDALGetRasterXSize(self.c_dataset) } as isize;
        let size_y = unsafe { gdal::GDALGetRasterYSize(self.c_dataset) } as isize;
        return (size_x, size_y);
    }

    pub fn driver(&self) -> Driver {
        unsafe {
            let c_driver = gdal::GDALGetDatasetDriver(self.c_dataset);
            return Driver::_with_c_ptr(c_driver);
        };
    }

    pub fn count(&self) -> isize {
        return unsafe { gdal::GDALGetRasterCount(self.c_dataset) } as isize;
    }

    pub fn projection(&self) -> String {
        let rv = unsafe { gdal::GDALGetProjectionRef(self.c_dataset) };
        return _string(rv);
    }

    pub fn set_projection(&self, projection: &str) {
        let c_projection = CString::from_slice(projection.as_bytes());
        unsafe { gdal::GDALSetProjection(self.c_dataset, c_projection.as_ptr()) };
    }

    pub fn set_geo_transform(&self, tr: &[f64]) {
        assert_eq!(tr.len(), 6);
        let rv = unsafe {
            gdal::GDALSetGeoTransform(self.c_dataset, tr.as_ptr())
        } as isize;
        assert!(rv == 0);
    }

    pub fn geo_transform(&self) -> Vec<f64> {
        let mut tr: Vec<c_double> = Vec::with_capacity(6);
        for _ in range(0is, 6) { tr.push(0.0); }
        let rv = unsafe {
            gdal::GDALGetGeoTransform(
                self.c_dataset,
                tr.as_mut_ptr()
            )
        } as isize;
        assert!(rv == 0);
        return tr;
    }

    pub fn create_copy(
        &self,
        driver: Driver,
        filename: &str
    ) -> Option<RasterDataset> {
        use std::ptr::null;
        let c_filename = CString::from_slice(filename.as_bytes());
        let c_dataset = unsafe { gdal::GDALCreateCopy(
                driver._c_ptr(),
                c_filename.as_ptr(),
                self.c_dataset,
                0,
                null(),
                null(),
                null()
            ) };
        return match c_dataset.is_null() {
            true  => None,
            false => Some(RasterDataset{c_dataset: c_dataset}),
        };
    }

    pub fn read_raster(&self,
        band_index: isize,
        window: Point<isize>,
        window_size: Point<usize>,
        size: Point<usize>
        ) -> ByteBuffer
    {
        let nbytes = size.x * size.y;
        let mut data: Vec<u8> = range(0, nbytes).map(|_| 0u8).collect();
        unsafe {
            let c_band = gdal::GDALGetRasterBand(self.c_dataset, band_index as c_int);
            let rv = gdal::GDALRasterIO(
                c_band,
                gdal::GF_READ,
                window.x as c_int,
                window.y as c_int,
                window_size.x as c_int,
                window_size.y as c_int,
                data.as_mut_ptr() as *const (),
                size.x as c_int,
                size.y as c_int,
                gdal::GDT_BYTE,
                0,
                0
            ) as isize;
            assert!(rv == 0);
        };
        return ByteBuffer{
            size: size,
            data: data,
        };
    }

    pub fn write_raster(
        &self,
        band_index: isize,
        window: Point<isize>,
        window_size: Point<usize>,
        buffer: ByteBuffer
    ) {
        assert_eq!(buffer.data.len(), buffer.size.x * buffer.size.y);
        unsafe {
            let c_band = gdal::GDALGetRasterBand(self.c_dataset, band_index as c_int);
            let rv = gdal::GDALRasterIO(
                c_band,
                gdal::GF_WRITE,
                window.x as c_int,
                window.y as c_int,
                window_size.x as c_int,
                window_size.y as c_int,
                buffer.data.as_ptr() as *const (),
                buffer.size.x as c_int,
                buffer.size.y as c_int,
                gdal::GDT_BYTE,
                0,
                0
            ) as isize;
            assert!(rv == 0);
        };
    }
}


pub fn open(path: &Path) -> Option<RasterDataset> {
    _register_drivers();
    let filename = path.as_str().unwrap();
    let c_filename = CString::from_slice(filename.as_bytes());
    let c_dataset = unsafe { gdal::GDALOpen(c_filename.as_ptr(), gdal::GA_READONLY) };
    return match c_dataset.is_null() {
        true  => None,
        false => Some(RasterDataset{c_dataset: c_dataset}),
    };
}


pub struct ByteBuffer {
    pub size: Point<usize>,
    pub data: Vec<u8>,
}
