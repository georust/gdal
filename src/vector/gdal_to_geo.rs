use std::convert::{TryFrom, TryInto};

use gdal_sys::{self, OGRwkbGeometryType};

use crate::errors::GdalError;
use crate::vector::Geometry;

impl TryFrom<Geometry> for geo_types::Geometry<f64> {
    type Error = GdalError;

    fn try_from(geo: Geometry) -> Result<geo_types::Geometry<f64>, Self::Error> {
        let geometry_type = geo.geometry_type();

        let ring = |n: usize| {
            let ring = unsafe { geo.get_unowned_geometry(n) };
            ring.try_into().map(|inner_geom| match inner_geom {
                geo_types::Geometry::LineString(r) => r,
                _ => panic!("Expected to get a LineString"),
            })
        };

        match geometry_type {
            OGRwkbGeometryType::wkbPoint => {
                let (x, y, _) = geo.get_point(0);
                Ok(geo_types::Geometry::Point(geo_types::Point(
                    geo_types::Coordinate { x, y },
                )))
            }
            OGRwkbGeometryType::wkbMultiPoint => {
                let point_count =
                    unsafe { gdal_sys::OGR_G_GetGeometryCount(geo.c_geometry()) } as usize;
                let coords = (0..point_count)
                    .map(|n| {
                        unsafe { geo.get_unowned_geometry(n) }
                            .try_into()
                            .map(|inner_geom| match inner_geom {
                                geo_types::Geometry::Point(p) => p,
                                _ => panic!("Expected to get a Point"),
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(geo_types::Geometry::MultiPoint(geo_types::MultiPoint(
                    coords,
                )))
            }
            OGRwkbGeometryType::wkbLineString => {
                let coords = geo
                    .get_point_vec()
                    .iter()
                    .map(|&(x, y, _)| geo_types::Coordinate { x, y })
                    .collect();
                Ok(geo_types::Geometry::LineString(geo_types::LineString(
                    coords,
                )))
            }
            OGRwkbGeometryType::wkbMultiLineString => {
                let string_count =
                    unsafe { gdal_sys::OGR_G_GetGeometryCount(geo.c_geometry()) } as usize;
                let strings = (0..string_count)
                    .map(|n| {
                        unsafe { geo.get_unowned_geometry(n) }
                            .try_into()
                            .map(|inner_geom| match inner_geom {
                                geo_types::Geometry::LineString(s) => s,
                                _ => panic!("Expected to get a LineString"),
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(geo_types::Geometry::MultiLineString(
                    geo_types::MultiLineString(strings),
                ))
            }
            OGRwkbGeometryType::wkbPolygon => {
                let ring_count =
                    unsafe { gdal_sys::OGR_G_GetGeometryCount(geo.c_geometry()) } as usize;
                let outer = ring(0)?;
                let holes = (1..ring_count).map(ring).collect::<Result<Vec<_>, _>>()?;
                Ok(geo_types::Geometry::Polygon(geo_types::Polygon::new(
                    outer, holes,
                )))
            }
            OGRwkbGeometryType::wkbMultiPolygon => {
                let string_count =
                    unsafe { gdal_sys::OGR_G_GetGeometryCount(geo.c_geometry()) } as usize;
                let strings = (0..string_count)
                    .map(|n| {
                        unsafe { geo.get_unowned_geometry(n) }
                            .try_into()
                            .map(|inner_geom| match inner_geom {
                                geo_types::Geometry::Polygon(s) => s,
                                _ => panic!("Expected to get a Polygon"),
                            })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(geo_types::Geometry::MultiPolygon(geo_types::MultiPolygon(
                    strings,
                )))
            }
            OGRwkbGeometryType::wkbGeometryCollection => {
                let item_count =
                    unsafe { gdal_sys::OGR_G_GetGeometryCount(geo.c_geometry()) } as usize;
                let geometry_list = (0..item_count)
                    .map(|n| unsafe { geo.get_unowned_geometry(n) }.try_into())
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(geo_types::Geometry::GeometryCollection(
                    geo_types::GeometryCollection(geometry_list),
                ))
            }
            _ => Err(GdalError::UnsupportedGdalGeometryType(geometry_type)),
        }
    }
}
