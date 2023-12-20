use crate::raster::GdalType;
use std::ops::{Index, IndexMut};
use std::slice::{Iter, IterMut};
use std::vec::IntoIter;

#[cfg(feature = "ndarray")]
use ndarray::Array2;

/// A 2-D array backed by it's `size` (cols, rows) and a row-major `Vec<T>` and it's dimensions.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Buffer<T> {
    shape: (usize, usize),
    data: Vec<T>,
}

impl<T: GdalType> Buffer<T> {
    /// Construct a new buffer from `size` (`(cols, rows)`) and `Vec<T>`.
    ///
    /// # Notes
    /// The elements of `shape` are in reverse order from what is used in `ndarray`.
    ///
    /// # Panics
    /// Will panic if `size.0 * size.1 != data.len()`.
    pub fn new(shape: (usize, usize), data: Vec<T>) -> Self {
        assert_eq!(
            shape.0 * shape.1,
            data.len(),
            "shape {}*{}={} does not match length {}",
            shape.0,
            shape.1,
            shape.0 * shape.1,
            data.len()
        );
        Buffer { shape, data }
    }

    /// Destructures `self` into constituent parts.
    pub fn into_shape_and_vec(self) -> ((usize, usize), Vec<T>) {
        (self.shape, self.data)
    }

    /// Gets the 2-d shape of the buffer.
    //
    /// Returns `(cols, rows)`
    ///
    /// # Notes
    /// The elements of `shape` are in reverse order from what is used in `ndarray`.
    pub fn shape(&self) -> (usize, usize) {
        self.shape
    }

    /// Get a slice over the buffer contents.
    pub fn data(&self) -> &[T] {
        self.data.as_slice()
    }

    /// Get a mutable slice over the buffer contents.
    pub fn data_mut(&mut self) -> &mut [T] {
        self.data.as_mut_slice()
    }

    /// Get the number of elements in the buffer
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Determine if the buffer has no elements.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    #[cfg(feature = "ndarray")]
    /// Convert `self` into an [`ndarray::Array2`].
    pub fn to_array(self) -> crate::errors::Result<Array2<T>> {
        // Array2 shape is (rows, cols) and Buffer shape is (cols in x-axis, rows in y-axis)
        Ok(Array2::from_shape_vec(
            (self.shape.1, self.shape.0),
            self.data,
        )?)
    }

    #[inline]
    fn vec_index_for(&self, coord: (usize, usize)) -> usize {
        if coord.0 >= self.shape.0 {
            panic!(
                "index out of bounds: buffer has {} columns but row {} was requested",
                self.shape.0, coord.0
            );
        }
        if coord.1 >= self.shape.1 {
            panic!(
                "index out of bounds: buffer has {} rows but row {} was requested",
                self.shape.1, coord.1
            );
        }
        coord.0 * self.shape.0 + coord.1
    }
}

impl<T: GdalType> Index<(usize, usize)> for Buffer<T> {
    type Output = T;
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.data[self.vec_index_for(index)]
    }
}

impl<T: GdalType> IndexMut<(usize, usize)> for Buffer<T> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        let idx = self.vec_index_for(index);
        &mut self.data[idx]
    }
}

impl<T: GdalType> IntoIterator for Buffer<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.into_iter()
    }
}

impl<'a, T: GdalType> IntoIterator for &'a Buffer<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter()
    }
}

impl<'a, T: GdalType> IntoIterator for &'a mut Buffer<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.data.iter_mut()
    }
}

pub type ByteBuffer = Buffer<u8>;

#[cfg(feature = "ndarray")]
impl<T: GdalType> TryFrom<Buffer<T>> for Array2<T> {
    type Error = crate::errors::GdalError;

    fn try_from(value: Buffer<T>) -> Result<Self, Self::Error> {
        value.to_array()
    }
}

#[cfg(feature = "ndarray")]
impl<T: GdalType + Copy> From<Array2<T>> for Buffer<T> {
    fn from(value: Array2<T>) -> Self {
        // Array2 shape is (rows, cols) and Buffer shape is (cols in x-axis, rows in y-axis)
        let shape = value.shape();
        let (cols, rows) = (shape[1], shape[0]);
        let data: Vec<T> = if value.is_standard_layout() {
            value.into_raw_vec()
        } else {
            value.iter().copied().collect()
        };

        Buffer::new((cols, rows), data)
    }
}

#[cfg(feature = "ndarray")]
#[cfg(test)]
mod tests {
    use crate::raster::Buffer;
    use ndarray::{Array2, ShapeBuilder};

    #[test]
    fn convert_to() {
        let b = Buffer::new((5, 10), (0..5 * 10).collect());
        let a = b.clone().to_array().unwrap();
        let expected = Array2::from_shape_fn((10, 5), |(y, x)| y as i32 * 5 + x as i32);
        assert_eq!(a, expected);
        let b2: Buffer<_> = a.into();
        assert_eq!(b, b2);
    }

    #[test]
    fn convert_from() {
        let a = Array2::from_shape_fn((10, 5), |(y, x)| y as i32 * 5 + x as i32);
        let b: Buffer<_> = a.clone().into();
        let expected = Buffer::new((5, 10), (0..5 * 10).collect());
        assert_eq!(b, expected);

        let a2 = b.to_array().unwrap();
        assert_eq!(a, a2);
    }

    #[test]
    fn shapes() {
        let s1 = (10, 5).into_shape().set_f(true);
        let s2 = (10, 5).into_shape().set_f(false);

        let a1 = Array2::from_shape_fn(s1, |(y, x)| y as i32 * 5 + x as i32);
        let a2 = Array2::from_shape_fn(s2, |(y, x)| y as i32 * 5 + x as i32);

        let expected = Buffer::new((5, 10), (0..5 * 10).collect());

        assert_eq!(a1, expected.clone().to_array().unwrap());

        let b1: Buffer<_> = a1.into();
        let b2: Buffer<_> = a2.into();

        assert_eq!(b1, expected);
        assert_eq!(b2, expected);
    }

    #[test]
    fn index() {
        let b = Buffer::new((5, 7), (0..5 * 7).collect());
        assert_eq!(b[(0, 0)], 0);
        assert_eq!(b[(1, 1)], 5 + 1);
        assert_eq!(b[(4, 5)], 4 * 5 + 5);

        let mut b = b;
        b[(2, 2)] = 99;
        assert_eq!(b[(2, 1)], 2 * 5 + 1);
        assert_eq!(b[(2, 2)], 99);
        assert_eq!(b[(2, 3)], 2 * 5 + 3);
    }

    #[test]
    fn iter() {
        // Iter on ref
        let b = Buffer::new((5, 7), (0..5 * 7).collect());
        let mut iter = (&b).into_iter();
        let _ = iter.next().unwrap();
        let v = iter.next().unwrap();
        assert_eq!(*v, b[(0, 1)]);

        // Iter on owned
        let mut iter = b.clone().into_iter();
        let _ = iter.next().unwrap();
        let v = iter.next().unwrap();
        assert_eq!(v, b[(0, 1)]);

        // Iter on mut
        let mut b = b;
        let mut iter = (&mut b).into_iter();
        let _ = iter.next().unwrap();
        let v = iter.next().unwrap();
        *v = 99;

        assert_eq!(99, b[(0, 1)]);
    }

    #[test]
    #[should_panic]
    fn index_bounds_panic() {
        let b = Buffer::new((5, 7), (0..5 * 7).collect());
        let _ = b[(5, 0)];
    }
}
