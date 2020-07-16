use super::srs::{SpatialRef, SpatialRefCommon};

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

    assert_eq!(true, spatial_ref1 == spatial_ref2);
    assert_eq!(false, spatial_ref2 == spatial_ref3);
    assert_eq!(true, spatial_ref4 == spatial_ref2);
    assert_eq!(true, spatial_ref5 == spatial_ref4);
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
fn auto_identify() {
    let mut spatial_ref = SpatialRef::from_wkt(
        r#"
        PROJCS["WGS_1984_UTM_Zone_32N",
            GEOGCS["GCS_WGS_1984",
                DATUM["D_WGS_1984",
                    SPHEROID["WGS_1984",6378137,298.257223563]],
                PRIMEM["Greenwich",0],
                UNIT["Degree",0.017453292519943295]],
            PROJECTION["Transverse_Mercator"],
            PARAMETER["latitude_of_origin",0],
            PARAMETER["central_meridian",9],
            PARAMETER["scale_factor",0.9996],
            PARAMETER["false_easting",500000],
            PARAMETER["false_northing",0],
            UNIT["Meter",1]]
    "#,
    )
    .unwrap();
    assert!(spatial_ref.auth_code().is_err());
    spatial_ref.auto_identify_epsg().unwrap();
    assert_eq!(spatial_ref.auth_code().unwrap(), 32632);
}
