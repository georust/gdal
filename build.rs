use libc::c_char;
use semver::Version;

#[cfg(docsrs)]
pub fn gdal_version_info(_key: &str) -> String {
    "GDAL 3.2.0, released 2222/02/22".to_string()
}

#[cfg(not(docsrs))]
pub fn gdal_version_info(key: &str) -> String {
    let c_key = std::ffi::CString::new(key.as_bytes()).unwrap();

    unsafe {
        let res_ptr = gdal_sys::GDALVersionInfo(c_key.as_ptr());
        let c_res = std::ffi::CStr::from_ptr(res_ptr);
        c_res.to_string_lossy().into_owned()
    }
}

#[cfg(not(docsrs))]
pub fn _gdal_create_geometry(wkt: &str) -> gdal_sys::OGRGeometryH {
    let c_wkt = std::ffi::CString::new(wkt).expect("Ooops, cannot create CString !");
    let mut c_wkt_ptr = c_wkt.as_ptr() as *mut c_char;
    let mut c_geom = std::ptr::null_mut();
    let rv =
        unsafe { gdal_sys::OGR_G_CreateFromWkt(&mut c_wkt_ptr, std::ptr::null_mut(), &mut c_geom) };
    if rv != gdal_sys::OGRErr::OGRERR_NONE {
        panic!(format!("Failed to create geometry from {}", wkt));
    }
    c_geom
}

#[cfg(not(docsrs))]
pub fn gdal_have_geos() -> bool {
    // Indirect way to test that gdal has geos
    // If gdal is not built with geos the function will fail with CPLE_NotSupported.
    // and return a null pointer
    let pnt1 = _gdal_create_geometry("POINT(10 20)");
    let pnt2 = _gdal_create_geometry("POINT(30 20)");
    let c_result = unsafe { gdal_sys::OGR_G_Union(pnt1, pnt2) };
    unsafe { gdal_sys::OGR_G_DestroyGeometry(pnt1) };
    unsafe { gdal_sys::OGR_G_DestroyGeometry(pnt2) };
    if !c_result.is_null() {
        unsafe { gdal_sys::OGR_G_DestroyGeometry(c_result) };
        true
    } else {
        false
    }
}

fn main() {
    let gdal_version_string = gdal_version_info("--version"); // This expects GDAL to repond with "GDAL Semver , RELEASE DATE"
    println!("GDAL version string: \"{}\"", gdal_version_string);

    let semver_substring = &gdal_version_string[4..gdal_version_string.find(',').unwrap_or(12)];
    println!("GDAL semver string: \"{}\"", semver_substring);

    let detected_version = Version::parse(semver_substring).expect("Could not parse gdal version!");

    if detected_version.major < 2 {
        panic!(
            "The GDAL crate requires a GDAL version >= 2.0.0. Found {}",
            detected_version.to_string()
        );
    }

    println!("cargo:rustc-cfg=gdal_{}", detected_version.major);
    println!(
        "cargo:rustc-cfg=gdal_{}_{}",
        detected_version.major, detected_version.minor
    );
    println!(
        "cargo:rustc-cfg=gdal_{}_{}_{}",
        detected_version.major, detected_version.minor, detected_version.patch
    );

    println!("cargo:rustc-cfg=major_is_{}", detected_version.major);
    println!("cargo:rustc-cfg=minor_is_{}", detected_version.minor);
    println!("cargo:rustc-cfg=patch_is_{}", detected_version.patch);

    // we only support GDAL >= 2.0.
    for major in 2..=detected_version.major {
        println!("cargo:rustc-cfg=major_ge_{}", major);
    }

    for minor in 0..=detected_version.minor {
        println!("cargo:rustc-cfg=minor_ge_{}", minor);
    }

    for patch in 0..=detected_version.patch {
        println!("cargo:rustc-cfg=patch_ge_{}", patch);
    }

    if gdal_have_geos() {
        println!("cargo:rustc-cfg=have_geos");
    }
}
