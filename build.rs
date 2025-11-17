use std::{env, str::FromStr};

fn main() {
    let gdal_version_string = env::var("DEP_GDAL_VERSION_NUMBER")
        .expect("The gdal-sys crate must emit the version of libgdal via cargo:version_number");
    println!("GDAL version string: \"{gdal_version_string}\"");

    // this version string is the result of:
    // #define GDAL_COMPUTE_VERSION(maj,min,rev) ((maj)*1000000+(min)*10000+(rev)*100)
    // so we can get the parts by doing the following
    let gdal_version = i64::from_str(&gdal_version_string)
        .expect("Could not convert gdal version string into number.");
    let major = gdal_version / 1000000;
    let minor = (gdal_version - major * 1000000) / 10000;
    let patch = (gdal_version - major * 1000000 - minor * 10000) / 100;

    if major < 3 || major == 4 && minor < 4 {
        panic!("The GDAL crate requires a GDAL version >= 3.4.0. Found {major}.{minor}.{patch}");
    }

    println!("cargo:rustc-cfg=gdal_{major}");
    println!("cargo:rustc-cfg=gdal_{major}_{minor}");

    println!("cargo:rustc-cfg=major_is_{major}");
    println!("cargo:rustc-cfg=minor_is_{minor}");

    for major in 3..=major {
        println!("cargo:rustc-cfg=major_ge_{major}");
    }

    for minor in 0..=minor {
        println!("cargo:rustc-cfg=minor_ge_{minor}");
    }
}
