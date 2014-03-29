pub struct Point<T> {
    x: T,
    y: T
}


pub fn Point<T>(x: T, y: T) -> Point<T> {
    return Point{x: x, y: y};
}

impl<T:Clone + Add<T,T>> Add<Point<T>, Point<T>> for Point<T> {
    fn add(&self, other: &Point<T>) -> Point<T> {
        return Point(self.x + other.x, self.y + other.y);
    }
}


#[test]
fn test_add() {
    let p1 = Point(2, 3);
    let p2 = Point(1, 5);
    let p3 = p1 + p2;
    assert_eq!((p3.x, p3.y), (3, 8));
}
