use std::str::FromStr;

#[cfg(feature = "docsrs")]
pub fn gdal_version_info(_key: &str) -> String {
    if std::env::var("DOCS_RS").is_err() {
        panic!(
            r#"The `docsrs` feature should not be enabled.
  If you are using the `--all-features` flag, try replacing it with `--features bindgen,array`.
  See https://github.com/georust/gdal/pull/241 for more details."#
        );
    };
    "3020000".to_string()
}

#[cfg(not(feature = "docsrs"))]
pub fn gdal_version_info(key: &str) -> String {
    let c_key = std::ffi::CString::new(key.as_bytes()).unwrap();

    unsafe {
        let res_ptr = gdal_sys::GDALVersionInfo(c_key.as_ptr());
        let c_res = std::ffi::CStr::from_ptr(res_ptr);
        c_res.to_string_lossy().into_owned()
    }
}

fn main() {
    let gdal_version_string = gdal_version_info("VERSION_NUM");
    println!("GDAL version string: \"{}\"", gdal_version_string);

    // this version string is the result of:
    // #define GDAL_COMPUTE_VERSION(maj,min,rev) ((maj)*1000000+(min)*10000+(rev)*100)
    // so we can get the parts by doing the following
    let gdal_version = i64::from_str(&gdal_version_string)
        .expect("Could not convert gdal version string into number.");
    let major = gdal_version / 1000000;
    let minor = (gdal_version - major * 1000000) / 10000;
    let patch = (gdal_version - major * 1000000 - minor * 10000) / 100;

    if major < 2 {
        panic!(
            "The GDAL crate requires a GDAL version >= 2.0.0. Found {}.{}.{}",
            major, minor, patch
        );
    }

    println!("cargo:rustc-cfg=gdal_{}", major);
    println!("cargo:rustc-cfg=gdal_{}_{}", major, minor);
    println!("cargo:rustc-cfg=gdal_{}_{}_{}", major, minor, patch);

    println!("cargo:rustc-cfg=major_is_{}", major);
    println!("cargo:rustc-cfg=minor_is_{}", minor);
    println!("cargo:rustc-cfg=patch_is_{}", patch);

    // we only support GDAL >= 2.0.
    for major in 2..=major {
        println!("cargo:rustc-cfg=major_ge_{}", major);
    }

    for minor in 0..=minor {
        println!("cargo:rustc-cfg=minor_ge_{}", minor);
    }

    for patch in 0..=patch {
        println!("cargo:rustc-cfg=patch_ge_{}", patch);
    }
}
