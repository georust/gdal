use std::num;


pub struct Point<T> {
    pub x: T,
    pub y: T
}


impl<T> Point<T> {
    pub fn new(x: T, y: T) -> Point<T> {
        return Point{x: x, y: y};
    }
}


impl<T:Clone + Add<T,T>> Add<Point<T>, Point<T>> for Point<T> {
    fn add(&self, other: &Point<T>) -> Point<T> {
        return Point::new(self.x + other.x, self.y + other.y);
    }
}


impl<T:Clone + Sub<T,T>> Sub<Point<T>, Point<T>> for Point<T> {
    fn sub(&self, other: &Point<T>) -> Point<T> {
        return Point::new(self.x - other.x, self.y - other.y);
    }
}


impl<T:Clone + Mul<T,T>> Point<T> {
    pub fn scale(&self, factor: T) -> Point<T> {
        return Point::new(self.x * factor, self.y * factor);
    }
}


#[cfg(test)]
mod test {
    use super::Point;


    #[test]
    fn test_add() {
        let p1 = Point::<int>::new(2, 3);
        let p2 = Point::<int>::new(1, 5);
        let p3 = p1 + p2;
        assert_eq!((p3.x, p3.y), (3, 8));
    }


    #[test]
    fn test_sub() {
        let p1 = Point::<int>::new(2, 3);
        let p2 = Point::<int>::new(1, 5);
        let p3 = p1 - p2;
        assert_eq!((p3.x, p3.y), (1, -2));
    }


    #[test]
    fn test_scale() {
        let p = Point::<int>::new(2, 3).scale(2);
        assert_eq!((p.x, p.y), (4, 6));
    }


    #[test]
    fn test_cast() {
        let pf = Point::<f64>::new(1.3, 2.9);
        let pi = pf.cast::<int>().unwrap();
        assert_eq!((pi.x, pi.y), (1, 2));
    }
}
