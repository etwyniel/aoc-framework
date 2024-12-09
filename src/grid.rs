use std::borrow::Cow;
use std::fmt::{Debug, Display};
use std::ops::{Deref, DerefMut, Index};
use std::{array, iter};

use crate::point::Point;
use crate::point::Point2;

#[derive(Clone)]
pub struct GridView<'a, T: Clone, const N: usize> {
    grid: Cow<'a, [T]>,
    stride: [usize; N],
    offset: Point<N>,
    size: Point<N>,
    // orientation: u8,
}

pub struct Grid<T: Clone + 'static, const N: usize>(GridView<'static, T, N>);

impl<T: Clone, const N: usize> Deref for Grid<T, N> {
    type Target = GridView<'static, T, N>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Clone, const N: usize> DerefMut for Grid<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Clone, const N: usize> AsRef<GridView<'static, T, N>> for Grid<T, N> {
    fn as_ref(&self) -> &GridView<'static, T, N> {
        &self.0
    }
}

impl<T: Clone, const N: usize> GridView<'_, T, N> {
    pub fn to_owned(self) -> Grid<T, N> {
        let GridView {
            grid,
            stride,
            offset,
            size,
        } = self;
        let grid = match grid {
            Cow::Owned(g) => Cow::Owned(g),
            Cow::Borrowed(b) => Cow::Owned(b.to_owned()),
        };
        Grid(GridView {
            grid,
            stride,
            offset,
            size,
        })
    }

    pub fn points_iter(&self) -> PointIter<N> {
        PointIter {
            size: self.size - Point::unit(),
            cur: Point::default(),
            done: false,
        }
    }

    pub fn data(&self) -> &[T] {
        self.grid.as_ref()
    }

    pub fn data_mut(&mut self) -> &mut [T] {
        self.grid.to_mut()
    }

    pub fn in_bounds(&self, p: Point<N>) -> bool {
        p.0.into_iter()
            .zip(self.size.0)
            .all(|(comp, size)| comp >= 0 && comp < size)
    }

    pub fn data_offset(&self, p: Point<N>) -> usize {
        let Point(components) = p + self.offset;
        components[0] as usize
            + components
                .into_iter()
                .zip(self.stride)
                .skip(1)
                .map(|(comp, stride)| comp * stride as isize)
                .sum::<isize>() as usize
    }

    pub fn offset_to_point(&self, mut off: usize) -> Point<N> {
        let mut p = Point::default();
        p.0.iter_mut()
            .zip(self.stride)
            .rev()
            .for_each(|(comp, stride)| {
                *comp = (off / stride) as isize;
                off %= stride;
            });
        p
    }

    pub fn get(&self, p: Point<N>) -> Option<&T> {
        if !self.in_bounds(p) {
            return None;
        }
        self.grid.get(self.data_offset(p))
    }

    pub fn set(&mut self, index: Point<N>, val: T) -> bool {
        if !self.in_bounds(index) {
            return false;
        }
        let pos = self.data_offset(index);
        if let Some(elem) = self.grid.to_mut().get_mut(pos) {
            *elem = val;
            true
        } else {
            false
        }
    }

    pub fn from_data(data: Vec<T>, stride: [usize; N]) -> Grid<T, N> {
        let mut size = [0; N];
        let mut len = data.len();
        size.iter_mut()
            .zip(stride)
            .rev()
            .for_each(|(size, stride)| {
                *size = (len / stride) as isize;
                len %= stride
            });
        Grid(GridView {
            grid: Cow::Owned(data),
            stride,
            offset: Point::default(),
            size: Point(size),
            // orientation: 3,
        })
    }
}

impl<T: Default + Clone, const N: usize> Grid<T, N> {
    pub fn from_size(size: [usize; N]) -> Grid<T, N> {
        let data: Vec<T> = vec![Default::default(); size.iter().product()];
        let stride = array::from_fn(|i| if i == 0 { 1 } else { size[i - 1] });
        let size = Point(array::from_fn(|i| size[i] as isize));
        Grid(GridView {
            grid: Cow::Owned(data),
            stride,
            offset: Point::default(),
            size,
        })
    }
}

impl<'a, T: Clone> GridView<'a, T, 2> {
    pub fn to_global(&self, p: Point2) -> Point2 {
        self.offset + p
    }

    pub fn to_local(&self, p: Point2) -> Point2 {
        p - self.offset
    }

    pub fn view(&self, offset: Point2, size: Point2) -> GridView<'_, T, 2> {
        GridView {
            grid: self.grid.clone(),
            stride: self.stride,
            offset,
            size,
            // orientation,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.grid.iter()
    }

    pub const fn size(&self) -> Point2 {
        self.size
    }
}

impl<'a, T: Clone> Index<Point2> for GridView<'a, T, 2> {
    type Output = T;
    fn index(&self, index: Point2) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<T: Clone + Default> Grid<T, 2> {
    pub fn from_data(data: Vec<T>, stride: usize) -> Grid<T, 2> {
        let h = data.len() / stride;
        Grid(GridView {
            grid: Cow::Owned(data),
            stride: [1, stride],
            offset: Point::default(),
            size: Point2::new(stride as isize, h as isize),
            // orientation: 3,
        })
    }

    pub fn from_lines<F: Fn(u8) -> T>(lines: Vec<String>, f: F) -> Grid<T, 2> {
        let w = lines.iter().map(|line| line.len()).max().unwrap();
        let f = &f;
        Grid::from_data(
            lines
                .iter()
                .flat_map(|line| {
                    line.bytes()
                        .map(f)
                        .chain(iter::repeat_with(T::default).take(w - line.len()))
                })
                .collect(),
            w,
        )
    }
}

impl Grid<u8, 2> {
    pub fn from_bytes(data: Vec<u8>) -> Self {
        let length = data.iter().position(|&b| b == b'\n').unwrap_or(data.len());
        let stride = length + 1;
        let height = (data.len() + 1) / stride;
        Grid(GridView {
            grid: data.into(),
            stride: [1, stride],
            offset: Point::default(),
            size: Point2::new(length as isize, height as isize),
        })
    }
}

impl<T: Clone + Display> GridView<'_, T, 2> {
    pub fn print(&self) {
        for y in 0..self.size.0[1] {
            for x in 0..self.size.0[0] {
                if let Some(elem) = self.get(Point2::new(x, y)) {
                    print!("{elem}")
                } else {
                    print!(" ")
                }
            }
            println!()
        }
    }
}

impl GridView<'_, u8, 2> {
    pub fn print_bytes(&self) {
        for y in 0..self.size.0[1] {
            for x in 0..self.size.0[0] {
                if let Some(&elem) = self.get(Point2::new(x, y)) {
                    print!("{}", elem as char)
                } else {
                    print!(" ")
                }
            }
            println!()
        }
    }
}

impl<T: Clone> Index<(isize, isize)> for GridView<'_, T, 2> {
    type Output = T;
    fn index(&self, (x, y): (isize, isize)) -> &T {
        let offset = self.data_offset(Point([x, y]));
        &self.grid[offset]
    }
}

impl<T: Clone + Display> Debug for GridView<'_, T, 2> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // writeln!(f)?;
        for y in 0..self.size.0[1] {
            for x in 0..self.size.0[0] {
                if let Some(elem) = self.get(Point2::new(x, y)) {
                    write!(f, "{elem}")?;
                } else {
                    write!(f, " ")?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

pub struct PointIter<const N: usize> {
    size: Point<N>,
    cur: Point<N>,
    done: bool,
}

impl<const N: usize> Iterator for PointIter<N> {
    type Item = Point<N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        let size = &self.size.0;
        let cur = self.cur;
        let next = &mut self.cur.0;
        let mut incremented = false;
        for (next, &size) in next.iter_mut().zip(size.iter()).rev() {
            if *next == size {
                *next = 0;
            } else {
                *next += 1;
                incremented = true;
                break;
            }
        }
        self.done = !incremented;
        Some(cur)
    }
}

#[cfg(test)]
mod tests {
    use super::Grid;
    use super::Point;
    #[test]
    fn test_points_iter() {
        let points = Grid::from_data(vec![0; 4], 2)
            .points_iter()
            .collect::<Vec<_>>();
        assert_eq!(
            &points,
            &[Point([0, 0]), Point([0, 1]), Point([1, 0]), Point([1, 1])]
        )
    }
}
