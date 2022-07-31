use std::clone::Clone;
use std::fmt;
use std::ops::{Index, IndexMut};

pub struct Array2d<T> {
    data: Vec<T>,
    height: usize,
    width: usize,
}

impl<T: Default + Clone> Array2d<T> {
    pub fn new(height: usize, width: usize) -> Array2d<T> {
        Self {
            data: vec![T::default(); height * width],
            height: height,
            width: width,
        }
    }

    pub fn apply<U: Default + Clone>(&self, func: fn(&T) -> U) -> Array2d<U> {
        let mut result = Array2d::new(self.height, self.width);

        for i in 0..self.data.len() {
            result.data[i] = func(&self.data[i]);
        }

        result
    }
}

impl<T> Index<(usize, usize)> for Array2d<T> {
    type Output = T;
    fn index(&self, idx: (usize, usize)) -> &Self::Output {
        let (y, x) = idx;
        let flat_idx = self.width * y + x;
        &self.data[flat_idx]
    }
}

impl<T> IndexMut<(usize, usize)> for Array2d<T> {
    fn index_mut(&mut self, idx: (usize, usize)) -> &mut Self::Output {
        let (y, x) = idx;
        let flat_idx = self.width * y + x;
        &mut self.data[flat_idx]
    }
}

impl<T: fmt::Display> fmt::Display for Array2d<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (idx, val) in self.data.iter().enumerate() {
            if idx % self.width == 0 {
                write!(f, "\n")?;
            }
            write!(f, "{}", val)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup() {
        let arr: Array2d<i32> = Array2d::new(10, 5);
        assert_eq!(arr.height, 10);
        assert_eq!(arr.width, 5);
    }

    #[test]
    fn test_array_read_value() {
        let arr: Array2d<i32> = Array2d::new(10, 5);
        for i in 0..10 {
            for j in 0..5 {
                assert_eq!(arr[(i, j)], 0);
            }
        }
    }

    #[test]
    fn test_apply() {
        let arr: Array2d<i32> = Array2d::new(10, 5).apply(|x: &i32| x + 1);
        assert_eq!(arr.height, 10);
        assert_eq!(arr.width, 5);
        for i in 0..10 {
            for j in 0..5 {
                assert_eq!(arr[(i, j)], 1);
            }
        }
    }

    #[test]
    fn test_array_write_value() {
        let mut arr: Array2d<i32> = Array2d::new(10, 5);
        arr[(1, 2)] = 1;

        for i in 0..10 {
            for j in 0..5 {
                if i == 1 && j == 2 {
                    assert_eq!(arr[(i, j)], 1);
                } else {
                    assert_eq!(arr[(i, j)], 0);
                }
            }
        }
    }
}
