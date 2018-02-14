use libc::c_int;

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[repr(C)]
pub enum OGRErr {
    OGRERR_NONE = 0,
    OGRERR_NOT_ENOUGH_DATA = 1,
    OGRERR_NOT_ENOUGH_MEMORY = 2,
    OGRERR_UNSUPPORTED_GEOMETRY_TYPE = 3,
    OGRERR_UNSUPPORTED_OPERATION = 4,
    OGRERR_CORRUPT_DATA = 5,
    OGRERR_FAILURE = 6,
    OGRERR_UNSUPPORTED_SRS = 7,
    OGRERR_INVALID_HANDLE = 8,
    OGRERR_NON_EXISTING_FEATURE = 9
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[allow(dead_code)]
#[allow(non_camel_case_types)]
#[repr(C)]
pub enum OGRFieldType {
    OFTInteger = 0,
    OFTIntegerList = 1,
    OFTReal = 2,
    OFTRealList = 3,
    OFTString = 4,
    OFTStringList = 5,
    OFTWideString = 6,
    OFTWideStringList = 7,
    OFTBinary = 8,
    OFTDate = 9,
    OFTTime = 10,
    OFTDateTime = 11,
    OFTInteger64 = 12,
    OFTInteger64List = 13,
    OFTMaxType = 14
}

pub const C_FALSE: c_int = 0;
pub const C_TRUE: c_int = 1;
