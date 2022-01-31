use super::srs::{CoordTransform, SpatialRef};
use crate::assert_almost_eq;
use crate::errors::GdalError;
use crate::vector::Geometry;

#[test]
fn from_wkt_to_proj4() {
    let spatial_ref = SpatialRef::from_wkt("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",7030]],TOWGS84[0,0,0,0,0,0,0],AUTHORITY[\"EPSG\",6326]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",8901]],UNIT[\"DMSH\",0.0174532925199433,AUTHORITY[\"EPSG\",9108]],AXIS[\"Lat\",NORTH],AXIS[\"Long\",EAST],AUTHORITY[\"EPSG\",4326]]").unwrap();
    assert_eq!(
        "+proj=longlat +ellps=WGS84 +towgs84=0,0,0,0,0,0,0 +no_defs",
        spatial_ref.to_proj4().unwrap().trim()
    );
    let spatial_ref = SpatialRef::from_definition("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",7030]],TOWGS84[0,0,0,0,0,0,0],AUTHORITY[\"EPSG\",6326]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",8901]],UNIT[\"DMSH\",0.0174532925199433,AUTHORITY[\"EPSG\",9108]],AXIS[\"Lat\",NORTH],AXIS[\"Long\",EAST],AUTHORITY[\"EPSG\",4326]]").unwrap();
    assert_eq!(
        "+proj=longlat +ellps=WGS84 +towgs84=0,0,0,0,0,0,0 +no_defs",
        spatial_ref.to_proj4().unwrap().trim()
    );
}

#[test]
fn from_proj4_to_wkt() {
    let spatial_ref = SpatialRef::from_proj4(
        "+proj=laea +lat_0=52 +lon_0=10 +x_0=4321000 +y_0=3210000 +ellps=GRS80 +units=m +no_defs",
    )
    .unwrap();
    // TODO: handle proj changes on lib level
    #[cfg(not(major_ge_3))]
    assert_eq!(spatial_ref.to_wkt().unwrap(), "PROJCS[\"unnamed\",GEOGCS[\"GRS 1980(IUGG, 1980)\",DATUM[\"unknown\",SPHEROID[\"GRS80\",6378137,298.257222101]],PRIMEM[\"Greenwich\",0],UNIT[\"degree\",0.0174532925199433]],PROJECTION[\"Lambert_Azimuthal_Equal_Area\"],PARAMETER[\"latitude_of_center\",52],PARAMETER[\"longitude_of_center\",10],PARAMETER[\"false_easting\",4321000],PARAMETER[\"false_northing\",3210000],UNIT[\"Meter\",1]]");
    #[cfg(major_ge_3)]
    assert_eq!(spatial_ref.to_wkt().unwrap(), "PROJCS[\"unknown\",GEOGCS[\"unknown\",DATUM[\"Unknown based on GRS80 ellipsoid\",SPHEROID[\"GRS 1980\",6378137,298.257222101,AUTHORITY[\"EPSG\",\"7019\"]]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",\"8901\"]],UNIT[\"degree\",0.0174532925199433,AUTHORITY[\"EPSG\",\"9122\"]]],PROJECTION[\"Lambert_Azimuthal_Equal_Area\"],PARAMETER[\"latitude_of_center\",52],PARAMETER[\"longitude_of_center\",10],PARAMETER[\"false_easting\",4321000],PARAMETER[\"false_northing\",3210000],UNIT[\"metre\",1,AUTHORITY[\"EPSG\",\"9001\"]],AXIS[\"Easting\",EAST],AXIS[\"Northing\",NORTH]]");
}

#[test]
fn from_epsg_to_wkt_proj4() {
    let spatial_ref = SpatialRef::from_epsg(4326).unwrap();
    let wkt = spatial_ref.to_wkt().unwrap();
    // TODO: handle proj changes on lib level
    #[cfg(not(major_ge_3))]
    assert_eq!("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",\"7030\"]],AUTHORITY[\"EPSG\",\"6326\"]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",\"8901\"]],UNIT[\"degree\",0.0174532925199433,AUTHORITY[\"EPSG\",\"9122\"]],AUTHORITY[\"EPSG\",\"4326\"]]", wkt);
    #[cfg(major_ge_3)]
    assert_eq!("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",\"7030\"]],AUTHORITY[\"EPSG\",\"6326\"]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",\"8901\"]],UNIT[\"degree\",0.0174532925199433,AUTHORITY[\"EPSG\",\"9122\"]],AXIS[\"Latitude\",NORTH],AXIS[\"Longitude\",EAST],AUTHORITY[\"EPSG\",\"4326\"]]", wkt);
    let proj4string = spatial_ref.to_proj4().unwrap();
    assert_eq!("+proj=longlat +datum=WGS84 +no_defs", proj4string.trim());
}

#[test]
fn from_esri_to_proj4() {
    let spatial_ref = SpatialRef::from_esri("GEOGCS[\"GCS_WGS_1984\",DATUM[\"D_WGS_1984\",SPHEROID[\"WGS_1984\",6378137,298.257223563]],PRIMEM[\"Greenwich\",0],UNIT[\"Degree\",0.017453292519943295]]").unwrap();
    let proj4string = spatial_ref.to_proj4().unwrap();
    assert_eq!("+proj=longlat +datum=WGS84 +no_defs", proj4string.trim());
}

#[test]
fn comparison() {
    let spatial_ref1 = SpatialRef::from_wkt("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",7030]],TOWGS84[0,0,0,0,0,0,0],AUTHORITY[\"EPSG\",6326]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",8901]],UNIT[\"DMSH\",0.0174532925199433,AUTHORITY[\"EPSG\",9108]],AXIS[\"Lat\",NORTH],AXIS[\"Long\",EAST],AUTHORITY[\"EPSG\",4326]]").unwrap();
    let spatial_ref2 = SpatialRef::from_epsg(4326).unwrap();
    let spatial_ref3 = SpatialRef::from_epsg(3025).unwrap();
    let spatial_ref4 = SpatialRef::from_proj4("+proj=longlat +datum=WGS84 +no_defs ").unwrap();
    let spatial_ref5 = SpatialRef::from_esri("GEOGCS[\"GCS_WGS_1984\",DATUM[\"D_WGS_1984\",SPHEROID[\"WGS_1984\",6378137,298.257223563]],PRIMEM[\"Greenwich\",0],UNIT[\"Degree\",0.017453292519943295]]").unwrap();

    assert!(spatial_ref1 == spatial_ref2);
    assert!(spatial_ref2 != spatial_ref3);
    assert!(spatial_ref4 == spatial_ref2);
    assert!(spatial_ref5 == spatial_ref4);
}

#[test]
fn transform_coordinates() {
    let spatial_ref1 = SpatialRef::from_wkt("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",7030]],TOWGS84[0,0,0,0,0,0,0],AUTHORITY[\"EPSG\",6326]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",8901]],UNIT[\"DMSH\",0.0174532925199433,AUTHORITY[\"EPSG\",9108]],AXIS[\"Lat\",NORTH],AXIS[\"Long\",EAST],AUTHORITY[\"EPSG\",4326]]").unwrap();
    let spatial_ref2 = SpatialRef::from_epsg(3035).unwrap();

    // TODO: handle axis order in tests
    #[cfg(major_ge_3)]
    spatial_ref1
        .set_axis_mapping_strategy(gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER);
    #[cfg(major_ge_3)]
    spatial_ref2
        .set_axis_mapping_strategy(gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER);

    let transform = CoordTransform::new(&spatial_ref1, &spatial_ref2).unwrap();
    let mut xs = [23.43, 23.50];
    let mut ys = [37.58, 37.70];
    let mut zs = [32.0, 20.0];
    transform
        .transform_coords(&mut xs, &mut ys, &mut zs)
        .unwrap();
    assert_almost_eq(xs[0], 5509543.1508097);
    assert_almost_eq(ys[0], 1716062.1916192223);
    assert_almost_eq(zs[0], 32.0);
}

#[test]
fn transform_ogr_geometry() {
    //let expected_value = "POLYGON ((5509543.150809700600803 1716062.191619219258428,5467122.000330002978444 1980151.204280239529908,5623571.028492723591626 2010213.310253676958382,5671834.921544363722205 1746968.078280254499987,5509543.150809700600803 1716062.191619219258428))";
    //let expected_value = "POLYGON ((5509543.15080969966948 1716062.191619222285226,5467122.000330002047122 1980151.204280242323875,5623571.028492721728981 2010213.31025367998518,5671834.921544362790883 1746968.078280256595463,5509543.15080969966948 1716062.191619222285226))";
    let expected_value = "POLYGON ((5509543.1508097 1716062.19161922,5467122.00033 1980151.20428024,5623571.02849272 2010213.31025368,5671834.92154436 1746968.07828026,5509543.1508097 1716062.19161922))";
    let mut geom = Geometry::from_wkt(
        "POLYGON((23.43 37.58, 23.43 40.0, 25.29 40.0, 25.29 37.58, 23.43 37.58))",
    )
    .unwrap();
    let spatial_ref1 = SpatialRef::from_proj4(
        "+proj=laea +lat_0=52 +lon_0=10 +x_0=4321000 +y_0=3210000 +ellps=GRS80 +units=m +no_defs",
    )
    .unwrap();
    let spatial_ref2 = SpatialRef::from_wkt("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",7030]],TOWGS84[0,0,0,0,0,0,0],AUTHORITY[\"EPSG\",6326]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",8901]],UNIT[\"DMSH\",0.0174532925199433,AUTHORITY[\"EPSG\",9108]],AXIS[\"Lat\",NORTH],AXIS[\"Long\",EAST],AUTHORITY[\"EPSG\",4326]]").unwrap();

    // TODO: handle axis order in tests
    #[cfg(major_ge_3)]
    spatial_ref1
        .set_axis_mapping_strategy(gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER);
    #[cfg(major_ge_3)]
    spatial_ref2
        .set_axis_mapping_strategy(gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER);

    let htransform = CoordTransform::new(&spatial_ref2, &spatial_ref1).unwrap();
    geom.transform_inplace(&htransform).unwrap();
    assert_eq!(expected_value, geom.wkt().unwrap());
}

#[test]
fn authority() {
    let spatial_ref = SpatialRef::from_epsg(4326).unwrap();
    assert_eq!(spatial_ref.auth_name().unwrap(), "EPSG".to_string());
    assert_eq!(spatial_ref.auth_code().unwrap(), 4326);
    assert_eq!(spatial_ref.authority().unwrap(), "EPSG:4326".to_string());
    let spatial_ref = SpatialRef::from_wkt("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",7030]],TOWGS84[0,0,0,0,0,0,0],AUTHORITY[\"EPSG\",6326]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",8901]],UNIT[\"DMSH\",0.0174532925199433,AUTHORITY[\"EPSG\",9108]],AXIS[\"Lat\",NORTH],AXIS[\"Long\",EAST],AUTHORITY[\"EPSG\",4326]]").unwrap();
    assert_eq!(spatial_ref.auth_name().unwrap(), "EPSG".to_string());
    assert_eq!(spatial_ref.auth_code().unwrap(), 4326);
    assert_eq!(spatial_ref.authority().unwrap(), "EPSG:4326".to_string());
    let spatial_ref = SpatialRef::from_wkt("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",7030]],TOWGS84[0,0,0,0,0,0,0],AUTHORITY[\"EPSG\",6326]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",8901]],UNIT[\"DMSH\",0.0174532925199433,AUTHORITY[\"EPSG\",9108]],AXIS[\"Lat\",NORTH],AXIS[\"Long\",EAST]]").unwrap();
    assert!(spatial_ref.auth_name().is_err());
    assert!(spatial_ref.auth_code().is_err());
    assert!(spatial_ref.authority().is_err());
    let spatial_ref = SpatialRef::from_proj4(
        "+proj=laea +lat_0=52 +lon_0=10 +x_0=4321000 +y_0=3210000 +ellps=GRS80 +units=m +no_defs",
    )
    .unwrap();
    assert!(spatial_ref.auth_name().is_err());
    assert!(spatial_ref.auth_code().is_err());
    assert!(spatial_ref.authority().is_err());
}

#[test]
fn failing_transformation() {
    let wgs84 = SpatialRef::from_epsg(4326).unwrap();
    let dhd_2 = SpatialRef::from_epsg(31462).unwrap();

    // TODO: handle axis order in tests
    #[cfg(major_ge_3)]
    wgs84.set_axis_mapping_strategy(gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER);
    #[cfg(major_ge_3)]
    dhd_2.set_axis_mapping_strategy(gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER);

    let mut x = [1979105.06, 0.0];
    let mut y = [5694052.67, 0.0];
    let mut z = [0.0, 0.0];

    let trafo = CoordTransform::new(&wgs84, &dhd_2).unwrap();
    let r = trafo.transform_coords(&mut x, &mut y, &mut z);
    assert!(r.is_err());

    let wgs84 = SpatialRef::from_epsg(4326).unwrap();
    let webmercator = SpatialRef::from_epsg(3857).unwrap();

    // TODO: handle axis order in tests
    #[cfg(major_ge_3)]
    wgs84.set_axis_mapping_strategy(gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER);
    #[cfg(major_ge_3)]
    webmercator
        .set_axis_mapping_strategy(gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER);

    let mut x = [1000000.0];
    let mut y = [1000000.0];

    let trafo = CoordTransform::new(&wgs84, &webmercator).unwrap();
    let r = trafo.transform_coords(&mut x, &mut y, &mut []);

    assert!(r.is_err());
    if let GdalError::InvalidCoordinateRange { .. } = r.unwrap_err() {
        // assert_eq!(msg, &Some("latitude or longitude exceeded limits".into()));
    } else {
        panic!("Wrong error type");
    }
}

#[test]
fn auto_identify() {
    // retreived from https://epsg.io/32632, but deleted the `AUTHORITY["EPSG","32632"]`
    let mut spatial_ref = SpatialRef::from_wkt(
        r#"
        PROJCS["WGS 84 / UTM zone 32N",
            GEOGCS["WGS 84",
                DATUM["WGS_1984",
                    SPHEROID["WGS 84",6378137,298.257223563,
                        AUTHORITY["EPSG","7030"]],
                    AUTHORITY["EPSG","6326"]],
                PRIMEM["Greenwich",0,
                    AUTHORITY["EPSG","8901"]],
                UNIT["degree",0.0174532925199433,
                    AUTHORITY["EPSG","9122"]],
                AUTHORITY["EPSG","4326"]],
            PROJECTION["Transverse_Mercator"],
            PARAMETER["latitude_of_origin",0],
            PARAMETER["central_meridian",9],
            PARAMETER["scale_factor",0.9996],
            PARAMETER["false_easting",500000],
            PARAMETER["false_northing",0],
            UNIT["metre",1,
                AUTHORITY["EPSG","9001"]],
            AXIS["Easting",EAST],
            AXIS["Northing",NORTH]]
    "#,
    )
    .unwrap();
    assert!(spatial_ref.auth_code().is_err());
    spatial_ref.auto_identify_epsg().unwrap();
    assert_eq!(spatial_ref.auth_code().unwrap(), 32632);
}

#[cfg(major_ge_3)]
#[test]
fn axis_mapping_strategy() {
    let spatial_ref = SpatialRef::from_epsg(4326).unwrap();
    assert_eq!(
        spatial_ref.axis_mapping_strategy(),
        gdal_sys::OSRAxisMappingStrategy::OAMS_AUTHORITY_COMPLIANT
    );
    spatial_ref
        .set_axis_mapping_strategy(gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER);
    assert_eq!(
        spatial_ref.axis_mapping_strategy(),
        gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER
    );
}

#[cfg(major_ge_3)]
#[test]
fn area_of_use() {
    let spatial_ref = SpatialRef::from_epsg(4326).unwrap();
    let area_of_use = spatial_ref.area_of_use().unwrap();
    assert_almost_eq(area_of_use.west_lon_degree, -180.0);
    assert_almost_eq(area_of_use.south_lat_degree, -90.0);
    assert_almost_eq(area_of_use.east_lon_degree, 180.0);
    assert_almost_eq(area_of_use.north_lat_degree, 90.0);
}

#[cfg(major_ge_3)]
#[test]
fn get_name() {
    let spatial_ref = SpatialRef::from_epsg(4326).unwrap();
    let name = spatial_ref.name().unwrap();
    assert_eq!(name, "WGS 84");
}

#[test]
fn get_units_epsg4326() {
    let spatial_ref = SpatialRef::from_epsg(4326).unwrap();

    let angular_units_name = spatial_ref.angular_units_name().unwrap();
    assert_eq!(angular_units_name.to_lowercase(), "degree");
    let to_radians = spatial_ref.angular_units();
    assert_almost_eq(to_radians, 0.01745329);
}

#[test]
fn get_units_epsg2154() {
    let spatial_ref = SpatialRef::from_epsg(2154).unwrap();
    let linear_units_name = spatial_ref.linear_units_name().unwrap();
    assert_eq!(linear_units_name.to_lowercase(), "metre");
    let to_meters = spatial_ref.linear_units();
    assert_almost_eq(to_meters, 1.0);
}

#[test]
fn predicats_epsg4326() {
    let spatial_ref_4326 = SpatialRef::from_epsg(4326).unwrap();
    assert!(spatial_ref_4326.is_geographic());
    assert!(!spatial_ref_4326.is_local());
    assert!(!spatial_ref_4326.is_projected());
    assert!(!spatial_ref_4326.is_compound());
    assert!(!spatial_ref_4326.is_geocentric());
    assert!(!spatial_ref_4326.is_vertical());

    #[cfg(all(major_ge_3, minor_ge_1))]
    assert!(!spatial_ref_4326.is_derived_geographic());
}

#[test]
fn predicats_epsg2154() {
    let spatial_ref_2154 = SpatialRef::from_epsg(2154).unwrap();
    assert!(!spatial_ref_2154.is_geographic());
    assert!(!spatial_ref_2154.is_local());
    assert!(spatial_ref_2154.is_projected());
    assert!(!spatial_ref_2154.is_compound());
    assert!(!spatial_ref_2154.is_geocentric());

    #[cfg(all(major_ge_3, minor_ge_1))]
    assert!(!spatial_ref_2154.is_derived_geographic());
}

//XXX Gdal 2 implementation is partial
#[cfg(major_ge_3)]
#[test]
fn crs_axis() {
    let spatial_ref = SpatialRef::from_epsg(4326).unwrap();

    #[cfg(all(major_ge_3, minor_ge_1))]
    assert_eq!(spatial_ref.axes_count(), 2);

    let orientation = spatial_ref.axis_orientation("GEOGCS", 0).unwrap();
    assert_eq!(orientation, gdal_sys::OGRAxisOrientation::OAO_North);
    assert!(spatial_ref.axis_name("GEOGCS", 0).is_ok());
    assert!(spatial_ref.axis_name("DO_NO_EXISTS", 0).is_err());
    assert!(spatial_ref.axis_orientation("DO_NO_EXISTS", 0).is_err());
}
