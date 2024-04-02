macro_rules! handle_ogr_driver {
    ($config: ident, $driver: literal) => {
        if cfg!(feature = $driver) {
            $config.define(format!("OGR_ENABLE_{}", $driver.to_ascii_uppercase()), "ON");
        } else {
            $config.define(
                format!("OGR_ENABLE_{}", $driver.to_ascii_uppercase()),
                "OFF",
            );
        }
    };
}

macro_rules! handle_gdal_driver {
    ($config: ident, $driver: literal) => {
        if cfg!(feature = $driver) {
            $config.define(
                format!("GDAL_ENABLE_{}", $driver.to_ascii_uppercase()),
                "ON",
            );
        } else {
            $config.define(
                format!("GDAL_ENABLE_{}", $driver.to_ascii_uppercase()),
                "OFF",
            );
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
    handle_ogr_driver!(config, "driver_avc");
    handle_ogr_driver!(config, "driver_cad");
    handle_ogr_driver!(config, "driver_csv");
    handle_ogr_driver!(config, "driver_dgn");
    handle_ogr_driver!(config, "driver_dxf");
    handle_ogr_driver!(config, "driver_edigeo");
    handle_ogr_driver!(config, "driver_flatgeobuf");
    handle_ogr_driver!(config, "driver_geoconcept");
    handle_ogr_driver!(config, "driver_geojson");
    handle_ogr_driver!(config, "driver_gmt");
    handle_ogr_driver!(config, "driver_gtfs");
    handle_ogr_driver!(config, "driver_jsonfg");
    handle_ogr_driver!(config, "driver_mapml");
    handle_ogr_driver!(config, "driver_openfilegdb");
    handle_ogr_driver!(config, "driver_pgdump");
    handle_ogr_driver!(config, "driver_ntf");
    handle_ogr_driver!(config, "driver_s57");
    handle_ogr_driver!(config, "driver_selafin");
    handle_ogr_driver!(config, "driver_shape");
    handle_ogr_driver!(config, "driver_sxf");
    handle_ogr_driver!(config, "driver_tab");
    handle_ogr_driver!(config, "driver_tiger");
    handle_ogr_driver!(config, "driver_vdv");
    handle_ogr_driver!(config, "driver_wasp");
    handle_ogr_driver!(config, "driver_idrisi");
    handle_ogr_driver!(config, "driver_sdts");
    handle_ogr_driver!(config, "driver_vrt");
    handle_ogr_driver!(config, "driver_mem");

    handle_gdal_driver!(config, "driver_aaigrid");
    handle_gdal_driver!(config, "driver_adrg");
    handle_gdal_driver!(config, "driver_aigrid");
    handle_gdal_driver!(config, "driver_airsar");
    handle_gdal_driver!(config, "driver_blx");
    handle_gdal_driver!(config, "driver_bmp");
    handle_gdal_driver!(config, "driver_bsb");
    handle_gdal_driver!(config, "driver_cals");
    handle_gdal_driver!(config, "driver_ceos");
    handle_gdal_driver!(config, "driver_coasp");
    handle_gdal_driver!(config, "driver_cosar");
    handle_gdal_driver!(config, "driver_ctg");
    handle_gdal_driver!(config, "driver_dimap");
    handle_gdal_driver!(config, "driver_dted");
    handle_gdal_driver!(config, "driver_elas");
    handle_gdal_driver!(config, "driver_envisat");
    handle_gdal_driver!(config, "driver_ers");
    handle_gdal_driver!(config, "driver_fit");
    handle_gdal_driver!(config, "driver_gff");
    handle_gdal_driver!(config, "driver_gif");
    handle_gdal_driver!(config, "driver_grib");
    handle_gdal_driver!(config, "driver_gsg");
    handle_gdal_driver!(config, "driver_gtiff");
    handle_gdal_driver!(config, "driver_gxf");
    handle_gdal_driver!(config, "driver_hf2");
    handle_gdal_driver!(config, "driver_hfa");
    handle_gdal_driver!(config, "driver_ilwis");
    handle_gdal_driver!(config, "driver_iris");
    handle_gdal_driver!(config, "driver_jaxapalsar");
    handle_gdal_driver!(config, "driver_jdem");
    handle_gdal_driver!(config, "driver_jpeg");
    handle_gdal_driver!(config, "driver_kmlsuperoverlay");
    handle_gdal_driver!(config, "driver_l1b");
    handle_gdal_driver!(config, "driver_leveller");
    handle_gdal_driver!(config, "driver_map");
    handle_gdal_driver!(config, "driver_mrf");
    handle_gdal_driver!(config, "driver_msgn");
    handle_gdal_driver!(config, "driver_ngsgeoid");
    handle_gdal_driver!(config, "driver_nift");
    handle_gdal_driver!(config, "driver_northwood");
    handle_gdal_driver!(config, "driver_ozi");
    handle_gdal_driver!(config, "driver_pcidsk");
    handle_gdal_driver!(config, "driver_pcraster");
    handle_gdal_driver!(config, "driver_png");
    handle_gdal_driver!(config, "driver_prf");
    handle_gdal_driver!(config, "driver_r");
    handle_gdal_driver!(config, "driver_raw");
    handle_gdal_driver!(config, "driver_rik");
    handle_gdal_driver!(config, "driver_rmf");
    handle_gdal_driver!(config, "driver_rs2");
    handle_gdal_driver!(config, "driver_safe");
    handle_gdal_driver!(config, "driver_saga");
    handle_gdal_driver!(config, "driver_sar_ceos");
    handle_gdal_driver!(config, "driver_sentinel2");
    handle_gdal_driver!(config, "driver_sgi");
    handle_gdal_driver!(config, "driver_sigdem");
    handle_gdal_driver!(config, "driver_srtmhgt");
    handle_gdal_driver!(config, "driver_stacit");
    handle_gdal_driver!(config, "driver_stacta");
    handle_gdal_driver!(config, "driver_terragen");
    handle_gdal_driver!(config, "driver_tga");
    handle_gdal_driver!(config, "driver_til");
    handle_gdal_driver!(config, "driver_tsx");
    handle_gdal_driver!(config, "driver_usgsdem");
    handle_gdal_driver!(config, "driver_xpm");
    handle_gdal_driver!(config, "driver_xyz");
    handle_gdal_driver!(config, "driver_zmap");
    handle_gdal_driver!(config, "driver_idrisi");
    handle_gdal_driver!(config, "driver_pds");
    handle_gdal_driver!(config, "driver_sdts");
    handle_gdal_driver!(config, "driver_vrt");
    handle_gdal_driver!(config, "driver_mem");

    if cfg!(feature = "driver_sqlite") {
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
    handle_ogr_driver!(config, "driver_gpkg");
    handle_ogr_driver!(config, "driver_vfk");

    if cfg!(feature = "driver_hdf5") {
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

    if cfg!(feature = "driver_netcdf") {
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

    handle_ogr_driver!(config, "driver_amigocloud");
    handle_ogr_driver!(config, "driver_carto");
    handle_ogr_driver!(config, "driver_daas");
    handle_ogr_driver!(config, "driver_eeda");
    handle_ogr_driver!(config, "driver_elastic");
    handle_ogr_driver!(config, "driver_ngw");
    handle_gdal_driver!(config, "driver_ogcapi");
    handle_gdal_driver!(config, "driver_plmosaic");
    handle_gdal_driver!(config, "driver_wcs");
    handle_gdal_driver!(config, "driver_wms");
    handle_gdal_driver!(config, "driver_wmts");

    if cfg!(feature = "driver_pg") {
        let pq_include = std::env::var("DEP_PQ_SYS_SRC_INCLUDE").expect("this is set by pq-src");
        let pq_lib = std::env::var("DEP_PQ_SYS_SRC_LIB_DIR").expect("this is set by pq-src");
        let pq_lib_path = std::path::PathBuf::from(&pq_lib);
        println!("cargo:rustc-link-search=native={}", pq_lib_path.display());
        let pq_lib_path = if pq_lib_path.join("libpq.a").exists() {
            pq_lib_path.join("libpq.a").display().to_string()
        } else if pq_lib_path.join("pq.lib").exists() {
            pq_lib_path.join("pq.lib").display().to_string()
        } else {
            panic!("Libpq not found in {pq_lib}");
        };

        println!("cargo:rustc-link-lib=static=pq");
        config
            .define("GDAL_USE_POSTGRESQL", "ON")
            .define("PostgreSQL_INCLUDE_DIR", pq_include)
            .define("PostgreSQL_LIBRARY_DEBUG", &pq_lib_path)
            .define("PostgreSQL_LIBRARY_RELEASE", &pq_lib_path)
            .define("OGR_ENABLE_DRIVER_PG", "ON");
    } else {
        config.define("GDAL_USE_POSTGRESQL", "OFF");
    }
    handle_gdal_driver!(config, "driver_postgis_raster");

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
