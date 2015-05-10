#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Geom {
    Point(Point),
}


#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
