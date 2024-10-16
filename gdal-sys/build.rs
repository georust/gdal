use semver::Version;

use pkg_config::Config;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[cfg(feature = "bindgen")]
pub fn write_bindings(include_paths: Vec<String>, out_path: &Path) {
    // To generate the bindings manually, use
    // bindgen --constified-enum-module ".*" --ctypes-prefix libc --allowlist-function "(CPL|CSL|GDAL|OGR|OSR|OCT|VSI).*" wrapper.h -- $(pkg-config --cflags-only-I gdal) -fretain-comments-from-system-headers
    // If you add a new pre-built version, make sure to bump the docs.rs version in main.

    let mut builder = bindgen::Builder::default()
        .size_t_is_usize(true)
        .header("wrapper.h")
        .constified_enum_module(".*")
        .ctypes_prefix("libc")
        .allowlist_function("CPL.*")
        .allowlist_function("CSL.*")
        .allowlist_function("GDAL.*")
        .allowlist_function("OGR.*")
        .allowlist_function("OSR.*")
        .allowlist_function("OCT.*")
        .allowlist_function("VSI.*");

    for path in include_paths {
        builder = builder
            .clang_arg("-I")
            .clang_arg(path)
            .clang_arg("-fretain-comments-from-system-headers");
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
    for e in fs::read_dir(lib_dir)? {
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
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("bindings.rs");

    // Hardcode a prebuilt binding version while generating docs.
    // Otherwise docs.rs will explode due to not actually having libgdal installed.
    if std::env::var("DOCS_RS").is_ok() || cfg!(feature = "bundled") {
        let version = Version::parse("3.9.0").expect("invalid version for docs.rs");
        println!(
            "cargo:rustc-cfg=gdal_sys_{}_{}_{}",
            version.major, version.minor, version.patch
        );

        // this version string is the result of:
        // #define GDAL_COMPUTE_VERSION(maj,min,rev) ((maj)*1000000+(min)*10000+(rev)*100)
        let gdal_version_number_string =
            version.major * 1_000_000 + version.minor * 10_000 + version.patch * 100;
        println!("cargo:version_number={}", gdal_version_number_string);

        let binding_path = PathBuf::from(format!(
            "prebuilt-bindings/gdal_{}_{}.rs",
            version.major, version.minor
        ));

        if !binding_path.exists() {
            panic!("Missing bindings for docs.rs (version {})", version);
        }

        std::fs::copy(&binding_path, &out_path).expect("Can't copy bindings to output directory");

        return;
    }

    println!("cargo:rerun-if-env-changed=GDAL_STATIC");
    println!("cargo:rerun-if-env-changed=GDAL_DYNAMIC");
    println!("cargo:rerun-if-env-changed=GDAL_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=GDAL_LIB_DIR");
    println!("cargo:rerun-if-env-changed=GDAL_HOME");
    println!("cargo:rerun-if-env-changed=GDAL_VERSION");

    let mut need_metadata = true;
    let mut lib_name = String::from("gdal");

    let mut prefer_static =
        env::var_os("GDAL_STATIC").is_some() && env::var_os("GDAL_DYNAMIC").is_none();

    let mut include_dir = env_dir("GDAL_INCLUDE_DIR");
    let mut lib_dir = env_dir("GDAL_LIB_DIR");
    let home_dir = env_dir("GDAL_HOME");
    let mut version = env::var_os("GDAL_VERSION")
        .map(|vs| vs.to_string_lossy().to_string())
        .and_then(|vs| Version::parse(vs.trim()).ok());

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

        println!("cargo:rustc-link-lib={link_type}={lib_name}");
        println!("cargo:rustc-link-search={}", lib_dir.to_str().unwrap());

        if !prefer_static {
            need_metadata = false;
        }
    }

    let mut include_paths = Vec::new();
    if let Some(ref dir) = include_dir {
        include_paths.push(dir.as_path().to_str().unwrap().to_string());
    }

    let gdal_pkg_config = Config::new()
        .statik(prefer_static)
        .cargo_metadata(need_metadata)
        .probe("gdal");

    if !found && cfg!(target_env = "msvc") && gdal_pkg_config.is_err() {
        panic!("windows-msvc requires gdal_i.lib to be present in either $GDAL_LIB_DIR or $GDAL_HOME\\lib.");
    }

    if let Ok(gdal) = &gdal_pkg_config {
        for dir in &gdal.include_paths {
            include_paths.push(dir.to_str().unwrap().to_string());
        }
        if version.is_none() {
            // development GDAL versions look like 3.7.2dev, which is not valid semver
            let mut version_string = gdal.version.trim().to_string();
            if let Some(idx) = version_string.rfind(|c: char| c.is_ascii_digit()) {
                if idx + 1 < version_string.len() && !version_string[idx + 1..].starts_with('-') {
                    version_string.insert(idx + 1, '-');
                }
            }

            if let Ok(pkg_version) = Version::parse(&version_string) {
                version.replace(pkg_version);
            }
        }
    }

    if let Some(gdal_version) = &version {
        // this version string is the result of:
        // #define GDAL_COMPUTE_VERSION(maj,min,rev) ((maj)*1000000+(min)*10000+(rev)*100)
        let gdal_version_number_string =
            gdal_version.major * 1_000_000 + gdal_version.minor * 10_000 + gdal_version.patch * 100;
        println!("cargo:version_number={}", gdal_version_number_string);
    }

    #[cfg(feature = "bindgen")]
    write_bindings(include_paths, &out_path);

    #[cfg(not(feature = "bindgen"))]
    {
        if let Some(version) = version {
            println!(
                "cargo:rustc-cfg=gdal_sys_{}_{}_{}",
                version.major, version.minor, version.patch
            );

            let binding_path = PathBuf::from(format!(
                "prebuilt-bindings/gdal_{}_{}.rs",
                version.major, version.minor
            ));
            if !binding_path.exists() {
                panic!("No pre-built bindings available for GDAL version {}.{}. Enable the `bindgen` feature of the `gdal` or `gdal-sys` crate to generate them during build.", version.major, version.minor);
            }

            std::fs::copy(&binding_path, &out_path)
                .expect("Can't copy bindings to output directory");
        } else if let Err(pkg_config_err) = &gdal_pkg_config {
            // Special case output for this common error
            if matches!(pkg_config_err, pkg_config::Error::Command { cause, .. } if cause.kind() == std::io::ErrorKind::NotFound)
            {
                panic!("Could not find `pkg-config` in your path. Please install it before building gdal-sys.");
            } else {
                panic!("Error while running `pkg-config`: {}", pkg_config_err);
            }
        } else {
            panic!("No GDAL version detected");
        }
    }
}
