use libc::{c_int, c_double, c_void};
use std::ffi::CString;
use std::path::Path;
use utils::_string;
use raster::{gdal, Driver};
use raster::driver::_register_drivers;
use raster::gdal_enums::{GDALRWFlag, GDALAccess, GDALDataType};
use raster::types::GdalType;
use gdal_major_object::MajorObject;
use metadata::Metadata;

pub type GeoTransform = [c_double; 6];

pub struct Dataset {
    c_dataset: *const c_void,
}

impl MajorObject for Dataset {
    unsafe fn get_gdal_object_ptr(&self) -> *const c_void {
        self.c_dataset
    }
}

impl Metadata for Dataset {}


impl Drop for Dataset {
    fn drop(&mut self) {
        unsafe { gdal::GDALClose(self.c_dataset); }
    }
}


impl Dataset {
    pub fn open(path: &Path) -> Option<Dataset> {
        _register_drivers();
        let filename = path.to_str().unwrap();
        let c_filename = CString::new(filename.as_bytes()).unwrap();
        let c_dataset = unsafe { gdal::GDALOpen(c_filename.as_ptr(), GDALAccess::GA_ReadOnly) };
        return match c_dataset.is_null() {
            true  => None,
            false => Some(Dataset{c_dataset: c_dataset}),
        };
    }

    pub unsafe fn _with_c_ptr(c_dataset: *const c_void) -> Dataset {
        return Dataset{c_dataset: c_dataset};
    }

    pub unsafe fn _c_ptr(&self) -> *const c_void {
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
        let c_projection = CString::new(projection.as_bytes()).unwrap();
        unsafe { gdal::GDALSetProjection(self.c_dataset, c_projection.as_ptr()) };
    }

    pub fn set_geo_transform(&self, tr: &GeoTransform) {
        assert_eq!(tr.len(), 6);
        let rv = unsafe {
            gdal::GDALSetGeoTransform(self.c_dataset, tr.as_ptr())
        } as isize;
        assert!(rv == 0);
    }

    pub fn geo_transform(&self) -> Option<GeoTransform> {
        let mut tr = GeoTransform::default();
        let rv = unsafe {
            gdal::GDALGetGeoTransform(
                self.c_dataset,
                tr.as_mut_ptr()
            )
        } as isize;

        // check if the dataset has a GeoTransform
        if rv != 0 {
            return None;
        }
        Some(tr)
    }

    pub fn create_copy(
        &self,
        driver: Driver,
        filename: &str
    ) -> Option<Dataset> {
        use std::ptr::null;
        let c_filename = CString::new(filename.as_bytes()).unwrap();
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
            false => Some(Dataset{c_dataset: c_dataset}),
        };
    }

    /// Read a 'Buffer<u8>' from a 'Dataset'.
    /// # Arguments
    /// * band_index - the band_index
    /// * window - the window position from top left
    /// * window_size - the window size (GDAL will interpolate data if window_size != buffer_size)
    /// * buffer_size - the desired size of the 'Buffer'
    pub fn read_raster(&self,
        band_index: isize,
        window: (isize, isize),
        window_size: (usize, usize),
        size: (usize, usize)
        ) -> ByteBuffer
    {
        self.read_raster_as::<u8>(
            band_index,
            window,
            window_size,
            size
        )
    }

    /// Read a full 'Dataset' as 'Buffer<T>'.
    /// # Arguments
    /// * band_index - the band_index
    pub fn read_full_raster_as<T: Copy + GdalType>(
        &self,
        band_index: isize,
    ) -> Buffer<T>
    {
        let size_x;
        let size_y;

        unsafe{
            size_x = gdal::GDALGetRasterXSize(self.c_dataset) as usize;
            size_y = gdal::GDALGetRasterYSize(self.c_dataset) as usize;
        }

        self.read_raster_as::<T>(
            band_index,
            (0, 0),
            (size_x, size_y),
            (size_y, size_y)
        )
    }

    /// Read a 'Buffer<T>' from a 'Dataset'. T implements 'GdalType'
    /// # Arguments
    /// * band_index - the band_index
    /// * window - the window position from top left
    /// * window_size - the window size (GDAL will interpolate data if window_size != buffer_size)
    /// * buffer_size - the desired size of the 'Buffer'
    pub fn read_raster_as<T: Copy + GdalType>(
        &self,
        band_index: isize,
        window: (isize, isize),
        window_size: (usize, usize),
        size: (usize, usize),
    ) -> Buffer<T>
    {
        let pixels = (size.0 * size.1) as usize;
        let mut data: Vec<T> = Vec::with_capacity(pixels);
        //let no_data:
        unsafe {
            let c_band = gdal::GDALGetRasterBand(self.c_dataset, band_index as c_int);
            let rv = gdal::GDALRasterIO(
                c_band,
                GDALRWFlag::GF_Read,
                window.0 as c_int,
                window.1 as c_int,
                window_size.0 as c_int,
                window_size.1 as c_int,
                data.as_mut_ptr() as *const c_void,
                size.0 as c_int,
                size.1 as c_int,
                T::gdal_type(),
                0,
                0
            ) as isize;
            assert!(rv == 0);
            data.set_len(pixels);
        };
        Buffer{
            size: size,
            data: data,
        }
    }

    /// Write a 'Buffer<T>' into a 'Dataset'.
    /// # Arguments
    /// * band_index - the band_index
    /// * window - the window position from top left
    /// * window_size - the window size (GDAL will interpolate data if window_size != Buffer.size)
    pub fn write_raster<T: GdalType+Copy>(
        &self,
        band_index: isize,
        window: (isize, isize),
        window_size: (usize, usize),
        buffer: Buffer<T>
    ) {
        assert_eq!(buffer.data.len(), buffer.size.0 * buffer.size.1);
        unsafe {
            let c_band = gdal::GDALGetRasterBand(self.c_dataset, band_index as c_int);
            let rv = gdal::GDALRasterIO(
                c_band,
                GDALRWFlag::GF_Write,
                window.0 as c_int,
                window.1 as c_int,
                window_size.0 as c_int,
                window_size.1 as c_int,
                buffer.data.as_ptr() as *const c_void,
                buffer.size.0 as c_int,
                buffer.size.1 as c_int,
                T::gdal_type(),
                0,
                0
            ) as isize;
            assert!(rv == 0);
        };
    }


    pub fn get_band_type(&self, band_index: isize) -> Option<GDALDataType> {

        let band_count = self.count();
        if band_index < 1 || band_count < band_index {
            return None
        }

        let gdal_type: c_int;
        unsafe{
            gdal_type = gdal::GDALGetRasterDataType(gdal::GDALGetRasterBand(self.c_dataset, band_index as c_int));
        }
        Some(GDALDataType::from_c_int(gdal_type))
    }
}

pub struct Buffer<T: GdalType> {
    pub size: (usize, usize),
    pub data: Vec<T>,
}

impl<T: GdalType> Buffer<T> {
    pub fn new(size: (usize, usize), data: Vec<T>) -> Buffer<T> {
        Buffer{size: size, data: data}
    }
}

pub type ByteBuffer = Buffer<u8>;
