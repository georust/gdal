use crate::raster::GdalType;

#[cfg(feature = "ndarray")]
use ndarray::Array2;

/// A 2-D array backed by it's `size` (cols, rows) and a row-major `Vec<T>` and it's dimensions.
#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Buffer<T> {
    pub size: (usize, usize),
    pub data: Vec<T>,
}

impl<T: GdalType> Buffer<T> {
    /// Construct a new buffer from `size` (`(cols, rows)`) and `Vec<T>`.
    ///
    /// # Panic
    /// Will panic if `size.0 * size.1 != data.len()`.
    pub fn new(size: (usize, usize), data: Vec<T>) -> Self {
        assert_eq!(
            size.0 * size.1,
            data.len(),
            "size {:?} does not match length {}",
            size,
            data.len()
        );
        Buffer { size, data }
    }

    #[cfg(feature = "ndarray")]
    /// Convert `self` into an [`ndarray::Array2`].
    pub fn to_array(self) -> crate::errors::Result<Array2<T>> {
        // Array2 shape is (rows, cols) and Buffer shape is (cols in x-axis, rows in y-axis)
        Ok(Array2::from_shape_vec(
            (self.size.1, self.size.0),
            self.data,
        )?)
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
        let (rows, cols) = (shape[0], shape[1]);
        let data = value
            .as_standard_layout()
            .iter()
            .copied()
            .collect::<Vec<T>>();
        Buffer::new((cols, rows), data)
    }
}

#[cfg(feature = "ndarray")]
#[cfg(test)]
mod tests {
    use crate::raster::Buffer;
    use ndarray::Array2;

    #[test]
    fn convert_to() {
        let b = Buffer::new((5, 10), (0..5 * 10).collect());
        let a = b.clone().to_array().unwrap();
        let b2: Buffer<_> = a.into();
        assert_eq!(b, b2);
    }

    #[test]
    fn convert_from() {
        let a = Array2::from_shape_fn((10, 5), |(y, x)| y as i32 * 10 + x as i32);
        let b: Buffer<_> = a.clone().into();
        let a2 = b.to_array().unwrap();
        assert_eq!(a, a2);
    }
}
