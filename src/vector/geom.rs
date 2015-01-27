use vector::Geometry;


#[derive(Clone, Copy, PartialEq, Show)]
pub enum Geom {
    Point(Point),
}


#[derive(Clone, Copy, PartialEq, Show)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
