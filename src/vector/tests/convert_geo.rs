use vector::{Geometry, ToGdal};
use geo;
use geo::ToGeo;

#[test]
fn test_import_export_point() {
    let wkt = "POINT (1 2)";
    let coord = geo::Coordinate{x: 1., y: 2.};
    let geo = geo::Geometry::Point(geo::Point(coord));

    assert_eq!(Geometry::from_wkt(wkt).unwrap().to_geo(), geo);
    assert_eq!(geo.to_gdal().unwrap().wkt().unwrap(), wkt);
}

#[test]
fn test_import_export_multipoint() {
    let wkt = "MULTIPOINT (0 0,0 1,1 2)";
    let coord = vec!(
        geo::Point(geo::Coordinate{x: 0., y: 0.}),
        geo::Point(geo::Coordinate{x: 0., y: 1.}),
        geo::Point(geo::Coordinate{x: 1., y: 2.}),
    );
    let geo = geo::Geometry::MultiPoint(geo::MultiPoint(coord));

    assert_eq!(Geometry::from_wkt(wkt).unwrap().to_geo(), geo);
    assert_eq!(geo.to_gdal().unwrap().wkt().unwrap(), wkt);
}

#[test]
fn test_import_export_linestring() {
    let wkt = "LINESTRING (0 0,0 1,1 2)";
    let coord = vec!(
        geo::Point(geo::Coordinate{x: 0., y: 0.}),
        geo::Point(geo::Coordinate{x: 0., y: 1.}),
        geo::Point(geo::Coordinate{x: 1., y: 2.}),
    );
    let geo = geo::Geometry::LineString(geo::LineString(coord));

    assert_eq!(Geometry::from_wkt(wkt).unwrap().to_geo(), geo);
    assert_eq!(geo.to_gdal().unwrap().wkt().unwrap(), wkt);
}

#[test]
fn test_import_export_multilinestring() {
    let wkt = "MULTILINESTRING ((0 0,0 1,1 2),(3 3,3 4,4 5))";
    let strings = vec!(
        geo::LineString(vec!(
            geo::Point(geo::Coordinate{x: 0., y: 0.}),
            geo::Point(geo::Coordinate{x: 0., y: 1.}),
            geo::Point(geo::Coordinate{x: 1., y: 2.}),
        )),
        geo::LineString(vec!(
            geo::Point(geo::Coordinate{x: 3., y: 3.}),
            geo::Point(geo::Coordinate{x: 3., y: 4.}),
            geo::Point(geo::Coordinate{x: 4., y: 5.}),
        )),
    );
    let geo = geo::Geometry::MultiLineString(geo::MultiLineString(strings));

    assert_eq!(Geometry::from_wkt(wkt).unwrap().to_geo(), geo);
    assert_eq!(geo.to_gdal().unwrap().wkt().unwrap(), wkt);
}

fn square(x0: isize, y0: isize, x1: isize, y1: isize) -> geo::LineString<f64> {
    geo::LineString(vec!(
        geo::Point(geo::Coordinate{x: x0 as f64, y: y0 as f64}),
        geo::Point(geo::Coordinate{x: x0 as f64, y: y1 as f64}),
        geo::Point(geo::Coordinate{x: x1 as f64, y: y1 as f64}),
        geo::Point(geo::Coordinate{x: x1 as f64, y: y0 as f64}),
        geo::Point(geo::Coordinate{x: x0 as f64, y: y0 as f64}),
    ))
}

#[test]
fn test_import_export_polygon() {
    let wkt = "POLYGON ((0 0,0 5,5 5,5 0,0 0),\
                        (1 1,1 2,2 2,2 1,1 1),\
                        (3 3,3 4,4 4,4 3,3 3))";
    let outer = square(0, 0, 5, 5);
    let holes = vec!(square(1, 1, 2, 2), square(3, 3, 4, 4));
    let geo = geo::Geometry::Polygon(geo::Polygon::new(outer, holes));

    assert_eq!(Geometry::from_wkt(wkt).unwrap().to_geo(), geo);
    assert_eq!(geo.to_gdal().unwrap().wkt().unwrap(), wkt);
}

#[test]
fn test_import_export_multipolygon() {
    let wkt = "MULTIPOLYGON (\
        ((0 0,0 5,5 5,5 0,0 0),\
         (1 1,1 2,2 2,2 1,1 1),\
         (3 3,3 4,4 4,4 3,3 3)),\
        ((4 4,4 9,9 9,9 4,4 4),\
         (5 5,5 6,6 6,6 5,5 5),\
         (7 7,7 8,8 8,8 7,7 7))\
        )";
    let multipolygon = geo::MultiPolygon(vec!(
        geo::Polygon::new(
            square(0, 0, 5, 5),
            vec!(square(1, 1, 2, 2), square(3, 3, 4, 4)),
        ),
        geo::Polygon::new(
            square(4, 4, 9, 9),
            vec!(square(5, 5, 6, 6), square(7, 7, 8, 8)),
        ),
    ));
    let geo = geo::Geometry::MultiPolygon(multipolygon);

    assert_eq!(Geometry::from_wkt(wkt).unwrap().to_geo(), geo);
    assert_eq!(geo.to_gdal().unwrap().wkt().unwrap(), wkt);
}

#[test]
fn test_import_export_geometrycollection() {
    let wkt = "GEOMETRYCOLLECTION (POINT (1 2),LINESTRING (0 0,0 1,1 2))";
    let coord = geo::Coordinate{x: 1., y: 2.};
    let point = geo::Geometry::Point(geo::Point(coord));
    let coords = vec!(
        geo::Point(geo::Coordinate{x: 0., y: 0.}),
        geo::Point(geo::Coordinate{x: 0., y: 1.}),
        geo::Point(geo::Coordinate{x: 1., y: 2.}),
    );
    let linestring = geo::Geometry::LineString(geo::LineString(coords));
    let collection = geo::GeometryCollection(vec!(point, linestring));
    let geo = geo::Geometry::GeometryCollection(collection);

    assert_eq!(Geometry::from_wkt(wkt).unwrap().to_geo(), geo);
    assert_eq!(geo.to_gdal().unwrap().wkt().unwrap(), wkt);
}
