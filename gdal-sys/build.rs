use pkg_config::Config;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[cfg(feature = "bindgen")]
pub fn write_bindings(include_paths: Vec<String>, out_path: &Path) {
    let mut builder = bindgen::Builder::default()
        .header("wrapper.h")
        .constified_enum_module(".*")
        .ctypes_prefix("libc")
        .whitelist_function("CPL.*")
        .whitelist_function("GDAL.*")
        .whitelist_function("OGR.*")
        .whitelist_function("OSR.*")
        .whitelist_function("OCT.*")
        .whitelist_function("VSI.*");

    for path in include_paths {
        builder = builder.clang_arg("-I");
        builder = builder.clang_arg(path);
    }

    builder
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_path)
        .expect("Unable to write bindings to file");
}

fn env_dir(var: &str) -> Option<PathBuf> {
    let dir = env::var_os(var).map(PathBuf::from);

    if let Some(ref dir) = dir {
        if !dir.exists() {
            panic!("{} was set to {}, which doesn't exist.", var, dir.display());
        }
    }

    dir
}

fn find_gdal_dll(lib_dir: &Path) -> io::Result<Option<String>> {
    for e in fs::read_dir(&lib_dir)? {
        let e = e?;
        let name = e.file_name();
        let name = name.to_str().unwrap();
        if name.starts_with("gdal") && name.ends_with(".dll") {
            return Ok(Some(String::from(name)));
        }
    }
    Ok(None)
}

fn main() {
    println!("cargo:rerun-if-env-changed=GDAL_STATIC");
    println!("cargo:rerun-if-env-changed=GDAL_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=GDAL_LIB_DIR");
    println!("cargo:rerun-if-env-changed=GDAL_HOME");

    let mut need_metadata = true;
    let mut lib_name = String::from("gdal");

    let mut prefer_static =
        env::var_os("GDAL_STATIC").is_some() && env::var_os("GDAL_DYNAMIC").is_none();

    let mut include_dir = env_dir("GDAL_INCLUDE_DIR");
    let mut lib_dir = env_dir("GDAL_LIB_DIR");
    let home_dir = env_dir("GDAL_HOME");

    let mut found = false;
    if cfg!(windows) {
        // first, look for a static library in $GDAL_LIB_DIR or $GDAL_HOME/lib
        // works in windows-msvc and windows-gnu
        if let Some(ref lib_dir) = lib_dir {
            let lib_path = lib_dir.join("gdal_i.lib");
            if lib_path.exists() {
                prefer_static = true;
                lib_name = String::from("gdal_i");
                found = true;
            }
        }
        if !found {
            if let Some(ref home_dir) = home_dir {
                let home_lib_dir = home_dir.join("lib");
                let lib_path = home_lib_dir.join("gdal_i.lib");
                if lib_path.exists() {
                    prefer_static = true;
                    lib_name = String::from("gdal_i");
                    lib_dir = Some(home_lib_dir);
                    found = true;
                }
            }
        }
        if !found {
            if cfg!(target_env = "msvc") {
                panic!("windows-gnu requires gdal_i.lib to be present in either $GDAL_LIB_DIR or $GDAL_HOME\\lib.");
            }

            // otherwise, look for a gdalxxx.dll in $GDAL_HOME/bin
            // works in windows-gnu
            if let Some(ref home_dir) = home_dir {
                let bin_dir = home_dir.join("bin");
                if bin_dir.exists() {
                    if let Some(name) = find_gdal_dll(&bin_dir).unwrap() {
                        prefer_static = false;
                        lib_dir = Some(bin_dir);
                        lib_name = name;
                    }
                }
            }
        }
    }

    if let Some(ref home_dir) = home_dir {
        if include_dir.is_none() {
            let dir = home_dir.join("include");
            if cfg!(feature = "bindgen") && !dir.exists() {
                panic!(
                    "bindgen was enabled, but GDAL_INCLUDE_DIR was not set and {} doesn't exist.",
                    dir.display()
                );
            }
            include_dir = Some(dir);
        }

        if lib_dir.is_none() {
            let dir = home_dir.join("lib");
            if !dir.exists() {
                panic!(
                    "GDAL_LIB_DIR was not set and {} doesn't exist.",
                    dir.display()
                );
            }
            lib_dir = Some(dir);
        }
    }

    if let Some(lib_dir) = lib_dir {
        let link_type = if prefer_static { "static" } else { "dylib" };

        println!("cargo:rustc-link-lib={}={}", link_type, lib_name);
        println!("cargo:rustc-link-search={}", lib_dir.to_str().unwrap());

        need_metadata = false;
    }

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");

    let mut include_paths = Vec::new();
    if let Some(ref dir) = include_dir {
        include_paths.push(dir.as_path().to_str().unwrap().to_string());
    }

    let gdal = Config::new()
        .statik(prefer_static)
        .cargo_metadata(need_metadata)
        .probe("gdal");

    if let Ok(gdal) = gdal {
        for dir in gdal.include_paths {
            include_paths.push(dir.to_str().unwrap().to_string());
        }
    }

    #[cfg(feature = "bindgen")]
    write_bindings(include_paths, &out_path);

    #[cfg(not(feature = "bindgen"))]
    {
        let prebuilt_paths = &[
            #[cfg(feature = "min_gdal_version_1_11")]
            "prebuilt-bindings/gdal_1.11.rs",
            #[cfg(feature = "min_gdal_version_2_0")]
            "prebuilt-bindings/gdal_2.0.rs",
            #[cfg(feature = "min_gdal_version_2_1")]
            "prebuilt-bindings/gdal_2.1.rs",
            #[cfg(feature = "min_gdal_version_2_2")]
            "prebuilt-bindings/gdal_2.2.rs",
            #[cfg(feature = "min_gdal_version_2_3")]
            "prebuilt-bindings/gdal_2.3.rs",
            #[cfg(feature = "min_gdal_version_2_4")]
            "prebuilt-bindings/gdal_2.4.rs",
            #[cfg(feature = "min_gdal_version_3_0")]
            "prebuilt-bindings/gdal_3.0.rs",
        ];
        std::fs::copy(&prebuilt_paths[prebuilt_paths.len() - 1], &out_path)
            .expect("Can't copy bindings to output directory");
    }
}
