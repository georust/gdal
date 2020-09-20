use crate::errors::ErrorKind;
use crate::{assert_almost_eq, Geometry, SpatialRef_3_0};
use crate::{CoordTransform, SpatialRef, SpatialRefCommon};

#[test]
fn from_proj4_to_wkt() {
    let spatial_ref = SpatialRef::from_proj4(
        "+proj=laea +lat_0=52 +lon_0=10 +x_0=4321000 +y_0=3210000 +ellps=GRS80 +units=m +no_defs",
    )
    .unwrap();

    assert_eq!(spatial_ref.to_wkt().unwrap(), "PROJCS[\"unknown\",GEOGCS[\"unknown\",DATUM[\"Unknown based on GRS80 ellipsoid\",SPHEROID[\"GRS 1980\",6378137,298.257222101,AUTHORITY[\"EPSG\",\"7019\"]]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",\"8901\"]],UNIT[\"degree\",0.0174532925199433,AUTHORITY[\"EPSG\",\"9122\"]]],PROJECTION[\"Lambert_Azimuthal_Equal_Area\"],PARAMETER[\"latitude_of_center\",52],PARAMETER[\"longitude_of_center\",10],PARAMETER[\"false_easting\",4321000],PARAMETER[\"false_northing\",3210000],UNIT[\"metre\",1,AUTHORITY[\"EPSG\",\"9001\"]],AXIS[\"Easting\",EAST],AXIS[\"Northing\",NORTH]]");
}

#[test]
fn from_epsg_to_wkt_proj4() {
    let spatial_ref = SpatialRef::from_epsg(4326).unwrap();
    let wkt = spatial_ref.to_wkt().unwrap();
    assert_eq!("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",\"7030\"]],AUTHORITY[\"EPSG\",\"6326\"]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",\"8901\"]],UNIT[\"degree\",0.0174532925199433,AUTHORITY[\"EPSG\",\"9122\"]],AXIS[\"Latitude\",NORTH],AXIS[\"Longitude\",EAST],AUTHORITY[\"EPSG\",\"4326\"]]", wkt);
    let proj4string = spatial_ref.to_proj4().unwrap();
    assert_eq!("+proj=longlat +datum=WGS84 +no_defs", proj4string.trim());
}

#[test]
fn transform_coordinates() {
    let spatial_ref1 = SpatialRef::from_wkt("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",7030]],TOWGS84[0,0,0,0,0,0,0],AUTHORITY[\"EPSG\",6326]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",8901]],UNIT[\"DMSH\",0.0174532925199433,AUTHORITY[\"EPSG\",9108]],AXIS[\"Lat\",NORTH],AXIS[\"Long\",EAST],AUTHORITY[\"EPSG\",4326]]").unwrap();
    let spatial_ref2 = SpatialRef::from_epsg(3035).unwrap();

    spatial_ref1
        .set_axis_mapping_strategy(gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER);
    spatial_ref2
        .set_axis_mapping_strategy(gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER);

    let transform = CoordTransform::new(&spatial_ref1, &spatial_ref2).unwrap();
    let mut xs = [23.43, 23.50];
    let mut ys = [37.58, 37.70];
    transform
        .transform_coords(&mut xs, &mut ys, &mut [0.0, 0.0])
        .unwrap();
    assert_almost_eq(xs[0], 5509543.1508097);
    assert_almost_eq(ys[0], 1716062.1916192223);
}

#[test]
fn transform_ogr_geometry() {
    //let expected_value = "POLYGON ((5509543.150809700600803 1716062.191619219258428,5467122.000330002978444 1980151.204280239529908,5623571.028492723591626 2010213.310253676958382,5671834.921544363722205 1746968.078280254499987,5509543.150809700600803 1716062.191619219258428))";
    //let expected_value = "POLYGON ((5509543.15080969966948 1716062.191619222285226,5467122.000330002047122 1980151.204280242323875,5623571.028492721728981 2010213.31025367998518,5671834.921544362790883 1746968.078280256595463,5509543.15080969966948 1716062.191619222285226))";
    let expected_value = "POLYGON ((5509543.1508097 1716062.19161922,5467122.00033 1980151.20428024,5623571.02849272 2010213.31025368,5671834.92154436 1746968.07828026,5509543.1508097 1716062.19161922))";
    let geom = Geometry::from_wkt(
        "POLYGON((23.43 37.58, 23.43 40.0, 25.29 40.0, 25.29 37.58, 23.43 37.58))",
    )
    .unwrap();
    let spatial_ref1 = SpatialRef::from_proj4(
        "+proj=laea +lat_0=52 +lon_0=10 +x_0=4321000 +y_0=3210000 +ellps=GRS80 +units=m +no_defs",
    )
    .unwrap();
    let spatial_ref2 = SpatialRef::from_wkt("GEOGCS[\"WGS 84\",DATUM[\"WGS_1984\",SPHEROID[\"WGS 84\",6378137,298.257223563,AUTHORITY[\"EPSG\",7030]],TOWGS84[0,0,0,0,0,0,0],AUTHORITY[\"EPSG\",6326]],PRIMEM[\"Greenwich\",0,AUTHORITY[\"EPSG\",8901]],UNIT[\"DMSH\",0.0174532925199433,AUTHORITY[\"EPSG\",9108]],AXIS[\"Lat\",NORTH],AXIS[\"Long\",EAST],AUTHORITY[\"EPSG\",4326]]").unwrap();

    spatial_ref1
        .set_axis_mapping_strategy(gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER);
    spatial_ref2
        .set_axis_mapping_strategy(gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER);

    let htransform = CoordTransform::new(&spatial_ref2, &spatial_ref1).unwrap();
    geom.transform_inplace(&htransform).unwrap();
    assert_eq!(expected_value, geom.wkt().unwrap());
}

#[test]
fn failing_transformation() {
    let wgs84 = SpatialRef::from_epsg(4326).unwrap();
    let dhd_2 = SpatialRef::from_epsg(31462).unwrap();

    wgs84.set_axis_mapping_strategy(gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER);
    dhd_2.set_axis_mapping_strategy(gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER);

    let mut x = [1979105.06, 0.0];
    let mut y = [5694052.67, 0.0];
    let mut z = [0.0, 0.0];

    let trafo = CoordTransform::new(&wgs84, &dhd_2).unwrap();
    let r = trafo.transform_coords(&mut x, &mut y, &mut z);
    assert_eq!(r.is_err(), true);

    let wgs84 = SpatialRef::from_epsg(4326).unwrap();
    let webmercator = SpatialRef::from_epsg(3857).unwrap();

    wgs84.set_axis_mapping_strategy(gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER);
    webmercator
        .set_axis_mapping_strategy(gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER);

    let mut x = [1000000.0];
    let mut y = [1000000.0];
    let mut z = [0.0, 0.0];

    let trafo = CoordTransform::new(&wgs84, &webmercator).unwrap();
    let r = trafo.transform_coords(&mut x, &mut y, &mut z);

    assert_eq!(r.is_err(), true);
    if let ErrorKind::InvalidCoordinateRange { .. } = r.unwrap_err().kind_ref() {
        // assert_eq!(msg, &Some("latitude or longitude exceeded limits".into()));
    } else {
        panic!("Wrong error type");
    }
}

#[test]
fn axis_mapping_strategy() {
    let spatial_ref = SpatialRef::from_epsg(4326).unwrap();
    assert_eq!(
        spatial_ref.get_axis_mapping_strategy(),
        gdal_sys::OSRAxisMappingStrategy::OAMS_AUTHORITY_COMPLIANT
    );
    spatial_ref
        .set_axis_mapping_strategy(gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER);
    assert_eq!(
        spatial_ref.get_axis_mapping_strategy(),
        gdal_sys::OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER
    );
}
