macro_rules! handle_ogr_driver {
    ($config: ident, $driver: literal) => {
        if cfg!(feature = $driver) {
            $config.define(concat!("OGR_ENABLE_", $driver), "ON");
        } else {
            $config.define(concat!("OGR_ENABLE_", $driver), "OFF");
        }
    };
}

macro_rules! handle_gdal_driver {
    ($config: ident, $driver: literal) => {
        if cfg!(feature = $driver) {
            $config.define(concat!("GDAL_ENABLE_", $driver), "ON");
        } else {
            $config.define(concat!("GDAL_ENABLE_", $driver), "OFF");
        }
    };
}

fn find_library(lib_name: &str, path: impl Into<std::path::PathBuf>) -> String {
    let path = path.into();
    if path.join("lib64").join(format!("lib{lib_name}.a")).exists() {
        path.join("lib64")
            .join(format!("lib{lib_name}.a"))
            .display()
            .to_string()
    } else if path.join("lib").join(format!("lib{lib_name}.a")).exists() {
        path.join("lib")
            .join(format!("lib{lib_name}.a"))
            .display()
            .to_string()
    } else if path.join("lib").join(format!("{lib_name}.lib")).exists() {
        path.join("lib")
            .join(format!("{lib_name}.lib"))
            .display()
            .to_string()
    } else {
        panic!("{lib_name} not found in {}", path.display());
    }
}

fn main() {
    let proj_root =
        std::path::PathBuf::from(std::env::var("DEP_PROJ_ROOT").expect("set by proj-sys"));
    let proj_library = if std::env::var("CARGO_CFG_TARGET_FAMILY").as_deref() == Ok("windows") {
        if proj_root.join("lib").join("proj_d.lib").exists() {
            proj_root
                .join("lib")
                .join("proj_d.lib")
                .display()
                .to_string()
        } else {
            proj_root.join("lib").join("proj.lib").display().to_string()
        }
    } else {
        find_library("proj", &proj_root)
    };

    let mut config = cmake::Config::new("source");

    config
        .define("GDAL_BUILD_OPTIONAL_DRIVERS", "OFF")
        .define("OGR_BUILD_OPTIONAL_DRIVERS", "OFF")
        .define("GDAL_USE_INTERNAL_LIBS", "ON")
        .define("GDAL_USE_EXTERNAL_LIBS", "OFF")
        .define("BUILD_SHARED_LIBS", "OFF")
        .define("BUILD_STATIC_LIBS", "ON")
        .define("BUILD_APPS", "OFF")
        .define("BUILD_DOCS", "OFF")
        .define("BUILD_TESTING", "OFF")
        .define("BUILD_GMOCK", "OFF")
        .define(
            "PROJ_INCLUDE_DIR",
            format!("{}/include", proj_root.display()),
        )
        .define("PROJ_LIBRARY", proj_library)
        .define("ACCEPT_MISSING_LINUX_FS_HEADER", "ON");
    // enable the gpkg driver

    // simple drivers without external dependencies
    handle_ogr_driver!(config, "DRIVER_AVC");
    handle_ogr_driver!(config, "DRIVER_CAD");
    handle_ogr_driver!(config, "DRIVER_CSV");
    handle_ogr_driver!(config, "DRIVER_DGN");
    handle_ogr_driver!(config, "DRIVER_DXF");
    handle_ogr_driver!(config, "DRIVER_EDIGEO");
    handle_ogr_driver!(config, "DRIVER_FLATGEOBUF");
    handle_ogr_driver!(config, "DRIVER_GEOCONCEPT");
    handle_ogr_driver!(config, "DRIVER_GEOJSON");
    handle_ogr_driver!(config, "DRIVER_GMT");
    handle_ogr_driver!(config, "DRIVER_GTFS");
    handle_ogr_driver!(config, "DRIVER_JSONFG");
    handle_ogr_driver!(config, "DRIVER_MAPML");
    handle_ogr_driver!(config, "DRIVER_OPENFILEGDB");
    handle_ogr_driver!(config, "DRIVER_PGDUMP");
    handle_ogr_driver!(config, "DRIVER_NTF");
    handle_ogr_driver!(config, "DRIVER_S57");
    handle_ogr_driver!(config, "DRIVER_SELAFIN");
    handle_ogr_driver!(config, "DRIVER_SHAPE");
    handle_ogr_driver!(config, "DRIVER_SXF");
    handle_ogr_driver!(config, "DRIVER_TAB");
    handle_ogr_driver!(config, "DRIVER_TIGER");
    handle_ogr_driver!(config, "DRIVER_VDV");
    handle_ogr_driver!(config, "DRIVER_WASP");
    handle_ogr_driver!(config, "DRIVER_IDRISI");
    handle_ogr_driver!(config, "DRIVEr_PDS");
    handle_ogr_driver!(config, "DRIVER_SDTS");
    handle_ogr_driver!(config, "DRIVER_VRT");
    handle_ogr_driver!(config, "DRIVER_MEM");

    handle_gdal_driver!(config, "DRIVER_AAIGRID");
    handle_gdal_driver!(config, "DRIVER_ADRG");
    handle_gdal_driver!(config, "DRIVER_AIGRID");
    handle_gdal_driver!(config, "DRIVER_AIRSAR");
    handle_gdal_driver!(config, "DRIVER_BLX");
    handle_gdal_driver!(config, "DRIVER_BMP");
    handle_gdal_driver!(config, "DRIVER_BSB");
    handle_gdal_driver!(config, "DRIVER_CALS");
    handle_gdal_driver!(config, "DRIVER_CEOS");
    handle_gdal_driver!(config, "DRIVER_COASP");
    handle_gdal_driver!(config, "DRIVER_COSAR");
    handle_gdal_driver!(config, "DRIVER_CTG");
    handle_gdal_driver!(config, "DRIVER_DIMAP");
    handle_gdal_driver!(config, "DRIVER_DTED");
    handle_gdal_driver!(config, "DRIVER_ELAS");
    handle_gdal_driver!(config, "DRIVER_ENVISAT");
    handle_gdal_driver!(config, "DRIVER_ERS");
    handle_gdal_driver!(config, "DRIVER_FIT");
    handle_gdal_driver!(config, "DRIVER_GFF");
    handle_gdal_driver!(config, "DRIVER_GIF");
    handle_gdal_driver!(config, "DRIVER_GRIB");
    handle_gdal_driver!(config, "DRIVER_GSG");
    handle_gdal_driver!(config, "DRIVER_GTIFF");
    handle_gdal_driver!(config, "DRIVER_GXF");
    handle_gdal_driver!(config, "DRIVER_HF2");
    handle_gdal_driver!(config, "DRIVER_HFA");
    handle_gdal_driver!(config, "DRIVER_ILWIS");
    handle_gdal_driver!(config, "DRIVER_IRIS");
    handle_gdal_driver!(config, "DRIVER_JAXAPALSAR");
    handle_gdal_driver!(config, "DRIVER_JDEM");
    handle_gdal_driver!(config, "DRIVER_JPEG");
    handle_gdal_driver!(config, "DRIVER_KMLSUPEROVERLAY");
    handle_gdal_driver!(config, "DRIVER_L1B");
    handle_gdal_driver!(config, "DRIVER_LEVELLER");
    handle_gdal_driver!(config, "DRIVER_MAP");
    handle_gdal_driver!(config, "DRIVER_MRF");
    handle_gdal_driver!(config, "DRIVER_MSGN");
    handle_gdal_driver!(config, "DRIVER_NGSGEOID");
    handle_gdal_driver!(config, "DRIVER_NIFT");
    handle_gdal_driver!(config, "DRIVER_NORTHWOOD");
    handle_gdal_driver!(config, "DRIVER_OZI");
    handle_gdal_driver!(config, "DRIVER_PCIDSK");
    handle_gdal_driver!(config, "DRIVER_PCRASTER");
    handle_gdal_driver!(config, "DRIVER_PNG");
    handle_gdal_driver!(config, "DRIVER_PRF");
    handle_gdal_driver!(config, "DRIVER_R");
    handle_gdal_driver!(config, "DRIVER_RAW");
    handle_gdal_driver!(config, "DRIVER_RIK");
    handle_gdal_driver!(config, "DRIVER_RMF");
    handle_gdal_driver!(config, "DRIVER_RS2");
    handle_gdal_driver!(config, "DRIVER_SAFE");
    handle_gdal_driver!(config, "DRIVER_SAGA");
    handle_gdal_driver!(config, "DRIVER_SAR_CEOS");
    handle_gdal_driver!(config, "DRIVER_SENTINEL2");
    handle_gdal_driver!(config, "DRIVER_SGI");
    handle_gdal_driver!(config, "DRIVER_SIGDEM");
    handle_gdal_driver!(config, "DRIVER_SRTMHGT");
    handle_gdal_driver!(config, "DRIVER_STACIT");
    handle_gdal_driver!(config, "DRIVER_STACTA");
    handle_gdal_driver!(config, "DRIVER_TERRAGEN");
    handle_gdal_driver!(config, "DRIVER_TGA");
    handle_gdal_driver!(config, "DRIVER_TIL");
    handle_gdal_driver!(config, "DRIVER_TSX");
    handle_gdal_driver!(config, "DRIVER_USGSDEM");
    handle_gdal_driver!(config, "DRIVER_XPM");
    handle_gdal_driver!(config, "DRIVER_XYZ");
    handle_gdal_driver!(config, "DRIVER_ZMAP");
    handle_gdal_driver!(config, "DRIVER_IDRISI");
    handle_gdal_driver!(config, "DRIVEr_PDS");
    handle_gdal_driver!(config, "DRIVER_SDTS");
    handle_gdal_driver!(config, "DRIVER_VRT");
    handle_gdal_driver!(config, "DRIVER_MEM");

    if cfg!(feature = "DRIVER_SQLITE") {
        let sqlite3_include_dir =
            std::env::var("DEP_SQLITE3_INCLUDE").expect("This is set by libsqlite3-sys");
        let sqlite3_lib_dir = std::env::var("DEP_SQLITE3_LIB_DIR").expect("set by libsqlite3-sys");

        config
            .define("GDAL_USE_SQLITE3", "ON")
            .define("SQLite3_INCLUDE_DIR", sqlite3_include_dir)
            .define("SQLite3_LIBRARY", format!("{sqlite3_lib_dir}/libsqlite3.a"))
            .define("OGR_ENABLE_DRIVER_SQLITE", "ON");
    } else {
        config.define("GDAL_USE_SQLITE3", "OFF");
    }
    // these drivers depend on sqlite
    handle_ogr_driver!(config, "DRIVER_GPKG");
    handle_ogr_driver!(config, "DRIVER_VFK");

    if cfg!(feature = "DRIVER_HDF5") {
        let hdf5_dir = std::env::var("DEP_HDF5SRC_ROOT").expect("This is set by hdf5-src");
        let hdf5_lib = std::env::var("DEP_HDF5SRC_LIBRARY").expect("This is set by hdf5-src");
        let hdf5_lib_dir = find_library(&hdf5_lib, &hdf5_dir);
        let p = std::path::PathBuf::from(&hdf5_lib_dir);
        let p = p.parent().unwrap();
        println!("cargo:rustc-link-search=native={}", p.display());
        println!("cargo:rustc-link-lib=static={hdf5_lib}");
        config
            .define("GDAL_USE_HDF5", "ON")
            .define("HDF5_C_COMPILER_EXECUTABLE", format!("{hdf5_dir}/bin/h5cc"))
            .define("HDF5_C_INCLUDE_DIR", format!("{hdf5_dir}/include"))
            .define("HDF5_hdf5_LIBRARY_DEBUG", &hdf5_lib_dir)
            .define("HDF5_hdf5_LIBRARY_RELEASE", &hdf5_lib_dir)
            .define("GDAL_ENABLE_DRIVER_HDF5", "ON")
            .define("HDF5_USE_STATIC_LIBRARIES", "ON");
    } else {
        config.define("GDAL_USE_HDF5", "OFF");
    }

    if cfg!(feature = "DRIVER_NETCDF") {
        let netcdf_root_dir =
            std::env::var("DEP_NETCDFSRC_ROOT").expect("This is set by netcdf-src");
        let hdf5_dir = std::env::var("DEP_HDF5SRC_ROOT").expect("This is set by hdf5-src");
        let hl_library = std::env::var("DEP_HDF5SRC_HL_LIBRARY").expect("This is set by hdf5-src");
        let netcdf_lib = find_library("netcdf", &netcdf_root_dir);
        let hl_library_path = find_library(&hl_library, hdf5_dir);
        let hl_library_path = std::path::PathBuf::from(hl_library_path);
        let hl_library_path = hl_library_path.parent().unwrap();

        let netcdf_library_path = std::path::PathBuf::from(&netcdf_lib);
        let netcdf_library_path = netcdf_library_path.parent().unwrap();
        println!(
            "cargo:rustc-link-search=native={}",
            netcdf_library_path.display()
        );
        println!("cargo:rustc-link-lib=static=netcdf");
        println!(
            "cargo:rustc-link-search=native={}",
            hl_library_path.display()
        );
        println!("cargo:rustc-link-lib=static={hl_library}");
        config
            .define("GDAL_USE_NETCDF", "ON")
            .define("NETCDF_INCLUDE_DIR", format!("{netcdf_root_dir}/include"))
            .define("NETCDF_LIBRARY", netcdf_lib)
            .define("GDAL_ENABLE_DRIVER_NETCDF", "ON");
    } else {
        config.define("GDAL_USE_NETCDF", "OFF");
    }

    if cfg!(feature = "curl-sys") {
        let curl_root = std::env::var("DEP_CURL_ROOT").expect("set from curl-sys");
        config
            .define("GDAL_USE_CURL", "ON")
            .define("CURL_INCLUDE_DIR", format!("{curl_root}/include"))
            .define("CURL_LIBRARY_DEBUG", format!("{curl_root}/build/libcurl.a"))
            .define(
                "CURL_LIBRARY_RELEASE",
                format!("{curl_root}/build/libcurl.a"),
            )
            .define("CURL_USE_STATIC_LIBS", "ON");
    } else {
        config.define("GDAL_USE_CURL", "OFF");
    }

    handle_ogr_driver!(config, "DRIVER_AMIGOCLOUD");
    handle_ogr_driver!(config, "DRIVER_CARTO");
    handle_ogr_driver!(config, "DRIVER_DAAS");
    handle_ogr_driver!(config, "DRIVER_EEDA");
    handle_ogr_driver!(config, "DRIVER_ELASTIC");
    handle_ogr_driver!(config, "DRIVER_NGW");
    handle_gdal_driver!(config, "DRIVER_OGCAPI");
    handle_gdal_driver!(config, "DRIVER_PLMOSAIC");
    handle_gdal_driver!(config, "DRIVER_WCS");
    handle_gdal_driver!(config, "DRIVER_WMS");
    handle_gdal_driver!(config, "DRIVER_WMTS");

    if cfg!(feature = "DRIVER_PG") {
        let pq_include = std::env::var("DEP_PQ_SYS_SRC_INCLUDE").expect("this is set by pq-src");
        let pq_lib = std::env::var("DEP_PQ_SYS_SRC_LIB_DIR").expect("this is set by pq-src");
        let pq_lib_path = std::path::PathBuf::from(&pq_lib);
        let pq_lib_path = if pq_lib_path.join("libpq.a").exists() {
            pq_lib_path.join("libpq.a").display().to_string()
        } else if pq_lib_path.join("pq.lib").exists() {
            pq_lib_path.join("pq.lib").display().to_string()
        } else {
            panic!("Libpq not found in {pq_lib}");
        };
        config
            .define("GDAL_USE_POSTGRESQL", "ON")
            .define("PostgreSQL_INCLUDE_DIR", pq_include)
            .define("PostgreSQL_LIBRARY_DEBUG", &pq_lib_path)
            .define("PostgreSQL_LIBRARY_RELEASE", &pq_lib_path)
            .define("OGR_ENABLE_DRIVER_PG", "ON");
    } else {
        config.define("GDAL_USE_POSTGRESQL", "OFF");
    }
    handle_gdal_driver!(config, "DRIVER_POSTGIS_RASTER");

    if cfg!(feature = "geos") {
        config.define("GDAL_USE_GEOS", "ON");
    } else {
        config.define("GDAL_USE_GEOS", "OFF");
    }

    if cfg!(feature = "geos_static") {
        let geos_root = std::env::var("DEP_GEOSSRC_ROOT").expect("this is set by geos-src");
        config.define("GEOS_INCLUDE_DIR", format!("{geos_root}/include"));
        let lib_path = find_library("geos", geos_root);
        config.define("GEOS_LIBRARY", lib_path);
    }

    if cfg!(target_env = "msvc") {
        // otherwise there are linking issues
        // because rust always links the
        // MSVC release runtime
        config.profile("Release");
        // see
        // https://github.com/OSGeo/PROJ/commit/6e9b324ab7bf5909df7e68409e060282db14fa54#diff-af8fe2f9d33a9c3408ff7683bfebd1e2334b4506f559add92406be3e150268fb
        config.cxxflag("-DPROJ_DLL=");
        // that windows library is somehow required
        println!("cargo:rustc-link-lib=Wbemuuid");
    }

    let res = config.build();

    // sometimes it's lib and sometimes it's lib64 and sometimes `build/lib`
    let lib_dir = res.join("lib64");
    println!(
        "cargo:rustc-link-search=native={}",
        lib_dir.to_str().unwrap()
    );
    let lib_dir = res.join("lib");
    println!(
        "cargo:rustc-link-search=native={}",
        lib_dir.to_str().unwrap()
    );
    let lib_dir = res.join("build/lib");
    println!(
        "cargo:rustc-link-search=native={}",
        lib_dir.to_str().unwrap()
    );

    //gdal likes to create gdal_d when configured as debug and on MSVC, so link to that one if it exists
    if res.join("lib").join("gdald.lib").exists() {
        println!("cargo:rustc-link-lib=static=gdald");
    } else {
        println!("cargo:rustc-link-lib=static=gdal");
    }
}
