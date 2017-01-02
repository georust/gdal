use libc::c_int;
use vector::{Geometry, ToGdal};
use geo;
use gdal_sys::ogr;

impl ToGdal for geo::Point {
    fn to_gdal(&self) -> Geometry {
        let mut geom = Geometry::empty(ogr::WKB_POINT);
        let &geo::Point(coordinate) = self;
        geom.set_point_2d(0, (coordinate.x, coordinate.y));
        return geom;
    }
}

impl ToGdal for geo::MultiPoint {
    fn to_gdal(&self) -> Geometry {
        let mut geom = Geometry::empty(ogr::WKB_MULTIPOINT);
        let &geo::MultiPoint(ref point_list) = self;
        for point in point_list.iter() {
            geom.add_geometry(point.to_gdal());
        }
        return geom;
    }
}

fn geometry_with_points(wkb_type: c_int, points: &geo::LineString) -> Geometry {
    let mut geom = Geometry::empty(wkb_type);
    let &geo::LineString(ref linestring) = points;
    for (i, &geo::Point(coordinate)) in linestring.iter().enumerate() {
        geom.set_point_2d(i, (coordinate.x, coordinate.y));
    }
    return geom;
}

impl ToGdal for geo::LineString {
    fn to_gdal(&self) -> Geometry {
        geometry_with_points(ogr::WKB_LINESTRING, self)
    }
}

impl ToGdal for geo::MultiLineString {
    fn to_gdal(&self) -> Geometry {
        let mut geom = Geometry::empty(ogr::WKB_MULTILINESTRING);
        let &geo::MultiLineString(ref point_list) = self;
        for point in point_list.iter() {
            geom.add_geometry(point.to_gdal());
        }
        return geom;
    }
}

impl ToGdal for geo::Polygon {
    fn to_gdal(&self) -> Geometry {
        let mut geom = Geometry::empty(ogr::WKB_POLYGON);
        let &geo::Polygon(ref outer, ref holes) = self;
        geom.add_geometry(geometry_with_points(ogr::WKB_LINEARRING, outer));
        for ring in holes.iter() {
            geom.add_geometry(geometry_with_points(ogr::WKB_LINEARRING, ring));
        }
        return geom;
    }
}

impl ToGdal for geo::MultiPolygon {
    fn to_gdal(&self) -> Geometry {
        let mut geom = Geometry::empty(ogr::WKB_MULTIPOLYGON);
        let &geo::MultiPolygon(ref polygon_list) = self;
        for polygon in polygon_list.iter() {
            geom.add_geometry(polygon.to_gdal());
        }
        return geom;
    }
}

impl ToGdal for geo::GeometryCollection {
    fn to_gdal(&self) -> Geometry {
        let mut geom = Geometry::empty(ogr::WKB_GEOMETRYCOLLECTION);
        let &geo::GeometryCollection(ref item_list) = self;
        for item in item_list.iter() {
            geom.add_geometry(item.to_gdal());
        }
        return geom;
    }
}

impl ToGdal for geo::Geometry {
    fn to_gdal(&self) -> Geometry {
        return match *self {
            geo::Geometry::Point(ref c) => c.to_gdal(),
            geo::Geometry::MultiPoint(ref c) => c.to_gdal(),
            geo::Geometry::LineString(ref c) => c.to_gdal(),
            geo::Geometry::MultiLineString(ref c) => c.to_gdal(),
            geo::Geometry::Polygon(ref c) => c.to_gdal(),
            geo::Geometry::MultiPolygon(ref c) => c.to_gdal(),
            geo::Geometry::GeometryCollection(ref c) => c.to_gdal(),
        }
    }
}
