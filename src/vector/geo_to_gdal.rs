use crate::errors::*;
use crate::vector::{Geometry, ToGdal};
use gdal_sys::OGRwkbGeometryType;
use geo_types;
use num_traits::Float;

impl<T> ToGdal for geo_types::Point<T>
where
    T: Float,
{
    fn to_gdal(&self) -> Result<Geometry> {
        let mut geom = Geometry::empty(OGRwkbGeometryType::wkbPoint)?;
        let &geo_types::Point(coordinate) = self;
        geom.set_point_2d(
            0,
            (
                coordinate.x.to_f64().ok_or(ErrorKind::CastToF64Error)?,
                coordinate.y.to_f64().ok_or(ErrorKind::CastToF64Error)?,
            ),
        );
        Ok(geom)
    }
}

impl<T> ToGdal for geo_types::MultiPoint<T>
where
    T: Float,
{
    fn to_gdal(&self) -> Result<Geometry> {
        let mut geom = Geometry::empty(OGRwkbGeometryType::wkbMultiPoint)?;
        let &geo_types::MultiPoint(ref point_list) = self;
        for point in point_list.iter() {
            geom.add_geometry(point.to_gdal()?)?;
        }
        Ok(geom)
    }
}

fn geometry_with_points<T>(
    wkb_type: OGRwkbGeometryType::Type,
    points: &geo_types::LineString<T>,
) -> Result<Geometry>
where
    T: Float,
{
    let mut geom = Geometry::empty(wkb_type)?;
    let &geo_types::LineString(ref linestring) = points;
    for (i, &coordinate) in linestring.iter().enumerate() {
        geom.set_point_2d(
            i,
            (
                coordinate.x.to_f64().ok_or(ErrorKind::CastToF64Error)?,
                coordinate.y.to_f64().ok_or(ErrorKind::CastToF64Error)?,
            ),
        );
    }
    Ok(geom)
}

impl<T> ToGdal for geo_types::Line<T>
where
    T: Float,
{
    fn to_gdal(&self) -> Result<Geometry> {
        let mut geom = Geometry::empty(OGRwkbGeometryType::wkbLineString)?;
        geom.set_point_2d(
            0,
            (
                self.start.x.to_f64().ok_or(ErrorKind::CastToF64Error)?,
                self.start.y.to_f64().ok_or(ErrorKind::CastToF64Error)?,
            ),
        );
        geom.set_point_2d(
            1,
            (
                self.end.x.to_f64().ok_or(ErrorKind::CastToF64Error)?,
                self.end.y.to_f64().ok_or(ErrorKind::CastToF64Error)?,
            ),
        );
        Ok(geom)
    }
}

impl<T> ToGdal for geo_types::LineString<T>
where
    T: Float,
{
    fn to_gdal(&self) -> Result<Geometry> {
        geometry_with_points(OGRwkbGeometryType::wkbLineString, self)
    }
}

impl<T> ToGdal for geo_types::MultiLineString<T>
where
    T: Float,
{
    fn to_gdal(&self) -> Result<Geometry> {
        let mut geom = Geometry::empty(OGRwkbGeometryType::wkbMultiLineString)?;
        let &geo_types::MultiLineString(ref point_list) = self;
        for point in point_list.iter() {
            geom.add_geometry(point.to_gdal()?)?;
        }
        Ok(geom)
    }
}

impl<T> ToGdal for geo_types::Polygon<T>
where
    T: Float,
{
    fn to_gdal(&self) -> Result<Geometry> {
        let mut geom = Geometry::empty(OGRwkbGeometryType::wkbPolygon)?;
        let exterior = self.exterior();
        let interiors = self.interiors();
        geom.add_geometry(geometry_with_points(
            OGRwkbGeometryType::wkbLinearRing,
            exterior,
        )?)?;
        for ring in interiors.iter() {
            geom.add_geometry(geometry_with_points(
                OGRwkbGeometryType::wkbLinearRing,
                ring,
            )?)?;
        }
        Ok(geom)
    }
}

impl<T> ToGdal for geo_types::MultiPolygon<T>
where
    T: Float,
{
    fn to_gdal(&self) -> Result<Geometry> {
        let mut geom = Geometry::empty(OGRwkbGeometryType::wkbMultiPolygon)?;
        let &geo_types::MultiPolygon(ref polygon_list) = self;
        for polygon in polygon_list.iter() {
            geom.add_geometry(polygon.to_gdal()?)?;
        }
        Ok(geom)
    }
}

impl<T> ToGdal for geo_types::GeometryCollection<T>
where
    T: Float,
{
    fn to_gdal(&self) -> Result<Geometry> {
        let mut geom = Geometry::empty(OGRwkbGeometryType::wkbGeometryCollection)?;
        let &geo_types::GeometryCollection(ref item_list) = self;
        for item in item_list.iter() {
            geom.add_geometry(item.to_gdal()?)?;
        }
        Ok(geom)
    }
}

impl<T> ToGdal for geo_types::Geometry<T>
where
    T: Float,
{
    fn to_gdal(&self) -> Result<Geometry> {
        match *self {
            geo_types::Geometry::Point(ref c) => c.to_gdal(),
            geo_types::Geometry::Line(ref c) => c.to_gdal(),
            geo_types::Geometry::LineString(ref c) => c.to_gdal(),
            geo_types::Geometry::Polygon(ref c) => c.to_gdal(),
            geo_types::Geometry::MultiPoint(ref c) => c.to_gdal(),
            geo_types::Geometry::MultiLineString(ref c) => c.to_gdal(),
            geo_types::Geometry::MultiPolygon(ref c) => c.to_gdal(),
            geo_types::Geometry::GeometryCollection(ref c) => c.to_gdal(),
        }
    }
}
