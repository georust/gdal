use vector::{Geometry, ToGdal};
use geo;
use gdal_sys::{OGRwkbGeometryType};
use errors::*;
use num_traits::{Float};

impl <T> ToGdal for geo::Point<T> where T: Float {
    fn to_gdal(&self) -> Result<Geometry> {
        let mut geom = Geometry::empty(OGRwkbGeometryType::wkbPoint)?;
        let &geo::Point(coordinate) = self;
        geom.set_point_2d(0, (coordinate.x.to_f64().ok_or("can't cast to f64")?, coordinate.y.to_f64().ok_or("can't cast to f64")?));
        Ok(geom)
    }
}

impl <T> ToGdal for geo::MultiPoint<T> where T: Float {
    fn to_gdal(&self) -> Result<Geometry> {
        let mut geom = Geometry::empty(OGRwkbGeometryType::wkbMultiPoint)?;
        let &geo::MultiPoint(ref point_list) = self;
        for point in point_list.iter() {
            geom.add_geometry(point.to_gdal()?)?;
        }
        Ok(geom)
    }
}

fn geometry_with_points<T>(wkb_type: OGRwkbGeometryType::Type, points: &geo::LineString<T>) -> Result<Geometry> where T: Float {
    let mut geom = Geometry::empty(wkb_type)?;
    let &geo::LineString(ref linestring) = points;
    for (i, &geo::Point(coordinate)) in linestring.iter().enumerate() {
        geom.set_point_2d(i, (coordinate.x.to_f64().ok_or("can't cast to f64")?, coordinate.y.to_f64().ok_or("can't cast to f64")?));
    }
    Ok(geom)
}

impl <T> ToGdal for geo::LineString<T> where T: Float {
    fn to_gdal(&self) -> Result<Geometry> {
        geometry_with_points(OGRwkbGeometryType::wkbLineString, self)
    }
}

impl <T> ToGdal for geo::MultiLineString<T> where T: Float {
    fn to_gdal(&self) -> Result<Geometry> {
        let mut geom = Geometry::empty(OGRwkbGeometryType::wkbMultiLineString)?;
        let &geo::MultiLineString(ref point_list) = self;
        for point in point_list.iter() {
            geom.add_geometry(point.to_gdal()?)?;
        }
        Ok(geom)
    }
}

impl <T> ToGdal for geo::Polygon<T> where T: Float {
    fn to_gdal(&self) -> Result<Geometry> {
        let mut geom = Geometry::empty(OGRwkbGeometryType::wkbPolygon)?;
        let &geo::Polygon{ref exterior, ref interiors} = self;
        geom.add_geometry(geometry_with_points(OGRwkbGeometryType::wkbLinearRing, exterior)?)?;
        for ring in interiors.iter() {
            geom.add_geometry(geometry_with_points(OGRwkbGeometryType::wkbLinearRing, ring)?)?;
        }
        Ok(geom)
    }
}

impl <T> ToGdal for geo::MultiPolygon<T> where T: Float {
    fn to_gdal(&self) -> Result<Geometry> {
        let mut geom = Geometry::empty(OGRwkbGeometryType::wkbMultiPolygon)?;
        let &geo::MultiPolygon(ref polygon_list) = self;
        for polygon in polygon_list.iter() {
            geom.add_geometry(polygon.to_gdal()?)?;
        }
        Ok(geom)
    }
}

impl <T> ToGdal for geo::GeometryCollection<T> where T: Float {
    fn to_gdal(&self) -> Result<Geometry> {
        let mut geom = Geometry::empty(OGRwkbGeometryType::wkbGeometryCollection)?;
        let &geo::GeometryCollection(ref item_list) = self;
        for item in item_list.iter() {
            geom.add_geometry(item.to_gdal()?)?;
        }
        Ok(geom)
    }
}

impl <T> ToGdal for geo::Geometry<T> where T: Float {
    fn to_gdal(&self) -> Result<Geometry> {
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
