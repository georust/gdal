use std::ops::{Add, Sub, Mul};


pub struct Point<T> {
    pub x: T,
    pub y: T
}


impl<T> Point<T> {
    pub fn new(x: T, y: T) -> Point<T> {
        return Point{x: x, y: y};
    }
}


impl<T:Clone + Add> Add for Point<T> where T: Add<Output = T> {
    type Output = Point<T>;

    fn add(self, other: Point<T>) -> Point<T> {
        return Point::new(self.x + other.x, self.y + other.y);
    }
}


impl<T:Clone + Sub> Sub for Point<T> where T: Sub<Output = T> {
    type Output = Point<T>;

    fn sub(self, other: Point<T>) -> Point<T> {
        return Point::new(self.x - other.x, self.y - other.y);
    }
}


impl<T:Clone + Mul> Point<T> where T: Mul<Output = T> {
    pub fn scale(&self, factor: T) -> Point<T> {
        let x = self.x.clone() * factor.clone();
        let y = self.y.clone() * factor;
        return Point::new(x, y);
    }
}


#[cfg(test)]
mod test {
    use super::Point;


    #[test]
    fn test_add() {
        let p1 = Point::<isize>::new(2, 3);
        let p2 = Point::<isize>::new(1, 5);
        let p3 = p1 + p2;
        assert_eq!((p3.x, p3.y), (3, 8));
    }


    #[test]
    fn test_sub() {
        let p1 = Point::<isize>::new(2, 3);
        let p2 = Point::<isize>::new(1, 5);
        let p3 = p1 - p2;
        assert_eq!((p3.x, p3.y), (1, -2));
    }


    #[test]
    fn test_scale() {
        let p = Point::<isize>::new(2, 3).scale(2);
        assert_eq!((p.x, p.y), (4, 6));
    }
}
