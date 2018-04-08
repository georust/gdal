use super::srs::{SpatialRef, CoordTransform};
use vector::Geometry;
use errors::ErrorKind;
use assert_almost_eq;

#[test]
fn from_wkt_to_proj4() {
    let spatial_ref = SpatialRef::from_wkt("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",7030]],TOWGS84[0,0,0,0,0,0,0],AUTHORITY[\"EPSG\",6326]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",8901]],UNIT[\"DMSH\",0.0174532925199433,AUTHORITY[\"EPSG\",9108]],AXIS[\"Lat\",NORTH],AXIS[\"Long\",EAST],AUTHORITY[\"EPSG\",4326]]").unwrap();
    assert_eq!("+proj=longlat +ellps=WGS84 +towgs84=0,0,0,0,0,0,0 +no_defs ", spatial_ref.to_proj4().unwrap());
    let spatial_ref = SpatialRef::from_definition("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",7030]],TOWGS84[0,0,0,0,0,0,0],AUTHORITY[\"EPSG\",6326]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",8901]],UNIT[\"DMSH\",0.0174532925199433,AUTHORITY[\"EPSG\",9108]],AXIS[\"Lat\",NORTH],AXIS[\"Long\",EAST],AUTHORITY[\"EPSG\",4326]]").unwrap();
    assert_eq!("+proj=longlat +ellps=WGS84 +towgs84=0,0,0,0,0,0,0 +no_defs ", spatial_ref.to_proj4().unwrap());
}

#[test]
fn from_proj4_to_wkt(){
    let spatial_ref = SpatialRef::from_proj4("+proj=laea +lat_0=52 +lon_0=10 +x_0=4321000 +y_0=3210000 +ellps=GRS80 +units=m +no_defs").unwrap();
    assert_eq!(spatial_ref.to_wkt().unwrap(), "PROJCS[\"unnamed\",GEOGCS[\"GRS 1980(IUGG, 1980)\",DATUM[\"unknown\",SPHEROID[\"GRS80\",6378137,298.257222101]],PRIMEM[\"Greenwich\",0],UNIT[\"degree\",0.0174532925199433]],PROJECTION[\"Lambert_Azimuthal_Equal_Area\"],PARAMETER[\"latitude_of_center\",52],PARAMETER[\"longitude_of_center\",10],PARAMETER[\"false_easting\",4321000],PARAMETER[\"false_northing\",3210000],UNIT[\"Meter\",1]]");
}

#[test]
fn from_epsg_to_wkt_proj4(){
    let spatial_ref = SpatialRef::from_epsg(4326).unwrap();
    let wkt = spatial_ref.to_wkt().unwrap();
    assert_eq!("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",\"7030\"]],AUTHORITY[\"EPSG\",\"6326\"]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",\"8901\"]],UNIT[\"degree\",0.0174532925199433,AUTHORITY[\"EPSG\",\"9122\"]],AUTHORITY[\"EPSG\",\"4326\"]]", wkt);
    let proj4string = spatial_ref.to_proj4().unwrap();
    assert_eq!("+proj=longlat +datum=WGS84 +no_defs ", proj4string);
}

#[test]
fn from_esri_to_proj4() {
    let spatial_ref = SpatialRef::from_esri("GEOGCS[\"GCS_WGS_1984\",DATUM[\"D_WGS_1984\",SPHEROID[\"WGS_1984\",6378137,298.257223563]],PRIMEM[\"Greenwich\",0],UNIT[\"Degree\",0.017453292519943295]]").unwrap();
    let proj4string = spatial_ref.to_proj4().unwrap();
    assert_eq!("+proj=longlat +datum=WGS84 +no_defs ", proj4string);
}

#[test]
fn comparison(){
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
fn transform_coordinates(){
    let spatial_ref1 = SpatialRef::from_wkt("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",7030]],TOWGS84[0,0,0,0,0,0,0],AUTHORITY[\"EPSG\",6326]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",8901]],UNIT[\"DMSH\",0.0174532925199433,AUTHORITY[\"EPSG\",9108]],AXIS[\"Lat\",NORTH],AXIS[\"Long\",EAST],AUTHORITY[\"EPSG\",4326]]").unwrap();
    let spatial_ref2 = SpatialRef::from_epsg(3035).unwrap();
    let transform = CoordTransform::new(&spatial_ref1, &spatial_ref2).unwrap();
    let mut xs = [23.43, 23.50];
    let mut ys = [37.58, 37.70];
    transform.transform_coords(&mut xs, &mut ys, &mut [0.0, 0.0]).unwrap();
    assert_almost_eq(xs[0], 5509543.1508097);
    assert_almost_eq(ys[0], 1716062.1916192223);
}

#[test]
fn transform_ogr_geometry(){
    //let expected_value = "POLYGON ((5509543.150809700600803 1716062.191619219258428,5467122.000330002978444 1980151.204280239529908,5623571.028492723591626 2010213.310253676958382,5671834.921544363722205 1746968.078280254499987,5509543.150809700600803 1716062.191619219258428))";
    //let expected_value = "POLYGON ((5509543.15080969966948 1716062.191619222285226,5467122.000330002047122 1980151.204280242323875,5623571.028492721728981 2010213.31025367998518,5671834.921544362790883 1746968.078280256595463,5509543.15080969966948 1716062.191619222285226))";
    let expected_value = "POLYGON ((5509543.1508097 1716062.19161922,5467122.00033 1980151.20428024,5623571.02849272 2010213.31025368,5671834.92154436 1746968.07828026,5509543.1508097 1716062.19161922))";
    let geom = Geometry::from_wkt("POLYGON((23.43 37.58, 23.43 40.0, 25.29 40.0, 25.29 37.58, 23.43 37.58))").unwrap();
    let spatial_ref1 = SpatialRef::from_proj4("+proj=laea +lat_0=52 +lon_0=10 +x_0=4321000 +y_0=3210000 +ellps=GRS80 +units=m +no_defs").unwrap();
    let spatial_ref2 = SpatialRef::from_wkt("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",7030]],TOWGS84[0,0,0,0,0,0,0],AUTHORITY[\"EPSG\",6326]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",8901]],UNIT[\"DMSH\",0.0174532925199433,AUTHORITY[\"EPSG\",9108]],AXIS[\"Lat\",NORTH],AXIS[\"Long\",EAST],AUTHORITY[\"EPSG\",4326]]").unwrap();
    let htransform = CoordTransform::new(&spatial_ref2, &spatial_ref1).unwrap();
    geom.transform_inplace(&htransform).unwrap();
    assert_eq!(expected_value, geom.wkt().unwrap());
}

#[test]
fn authority(){
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
    let spatial_ref = SpatialRef::from_proj4("+proj=laea +lat_0=52 +lon_0=10 +x_0=4321000 +y_0=3210000 +ellps=GRS80 +units=m +no_defs").unwrap();
    assert!(spatial_ref.auth_name().is_err());
    assert!(spatial_ref.auth_code().is_err());
    assert!(spatial_ref.authority().is_err());
}


#[test]
fn failing_transformation() {
    let wgs84 = SpatialRef::from_epsg(4326).unwrap();
    let dhd_2 = SpatialRef::from_epsg(31462).unwrap();

    let mut x = [1979105.06, 0.0];
    let mut y = [5694052.67, 0.0];
    let mut z = [0.0, 0.0];

    let trafo = CoordTransform::new(&wgs84, &dhd_2).unwrap();
    let r = trafo.transform_coords(&mut x, &mut y, &mut z);
    assert_eq!(r.is_err(), true);

    let wgs84 = SpatialRef::from_epsg(4326).unwrap();
    let webmercator = SpatialRef::from_epsg(3857).unwrap();

    let mut x = [1000000.0];
    let mut y = [1000000.0];
    let mut z = [0.0, 0.0];

    let trafo = CoordTransform::new(&wgs84, &webmercator).unwrap();
    let r = trafo.transform_coords(&mut x, &mut y, &mut z);

    assert_eq!(r.is_err(), true);
    if let &ErrorKind::InvalidCoordinateRange{ref msg, ..} = r.unwrap_err().kind_ref() {
        assert_eq!(msg, &Some("latitude or longitude exceeded limits".into()));
    } else {
        panic!("Wrong error type");
    }
}

#[test]
fn auto_identify() {
    let mut spatial_ref = SpatialRef::from_wkt(r#"
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
    "#).unwrap();
    assert!(spatial_ref.auth_code().is_err());
    spatial_ref.auto_identify_epsg().unwrap();
    assert_eq!(spatial_ref.auth_code().unwrap(), 32632);
}
