use std::borrow::Cow;
use std::fmt::{Debug, Display};
use std::iter;
use std::ops::{Deref, DerefMut, Index};

use crate::point::Point2;
use crate::{
    helpers::Direction,
    point::{ext::Point2d, Point},
};

#[derive(Clone)]
pub struct GridView<'a, T: Clone, const N: usize> {
    grid: Cow<'a, [T]>,
    stride: usize,
    offset: Point<N>,
    size: Point<N>,
    orientation: u8,
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

impl<'a, T: Clone, const N: usize> GridView<'a, T, N> {
    pub fn points_iter(&self) -> PointIter<N> {
        PointIter {
            size: self.size,
            i: 0,
        }
    }
}

impl<'a, T: Clone> GridView<'a, T, 2> {
    fn lastx(&self) -> isize {
        self.size.x() - 1
    }

    fn lasty(&self) -> isize {
        self.size.y() - 1
    }

    pub fn to_global(&self, p: Point2) -> Point2 {
        let x = p.x();
        let y = p.y();
        self.offset
            + match self.orientation {
                3 => Point2::new(x, y),
                0 => Point2::new(self.lasty() - y, x),
                1 => Point2::new(self.lastx() - x, self.lasty() - y),
                2 => Point2::new(y, self.lastx() - x),
                _ => unreachable!(),
            }
    }

    pub fn to_local(&self, p: Point2) -> Point2 {
        let p = p - self.offset;
        let x = p.x();
        let y = p.y();
        match self.orientation {
            3 => Point2::new(x, y),
            0 => Point2::new(y, self.lasty() - x),
            1 => Point2::new(self.lastx() - x, self.lasty() - y),
            2 => Point2::new(self.lastx() - y, x),
            _ => unreachable!(),
        }
    }

    pub fn view(&self, offset: Point2, size: Point2, orientation: u8) -> GridView<'_, T, 2> {
        GridView {
            grid: self.grid.clone(),
            stride: self.stride,
            offset,
            size,
            orientation,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.grid.iter()
    }

    pub fn get(&self, index: Point2) -> Option<&T> {
        if index.x() >= self.size.x()
            || index.y() >= self.size.y()
            || index.x() < 0
            || index.y() < 0
        {
            return None;
        }
        let pos = self.to_global(index);
        let i = pos.y() as usize * self.stride + pos.x() as usize;
        self.grid.get(i)
    }

    pub fn set(&mut self, index: Point2, val: T) -> bool {
        if index.x() >= self.size.x()
            || index.y() >= self.size.y()
            || index.x() < 0
            || index.y() < 0
        {
            return false;
        }
        let pos = self.to_global(index);
        let i = pos.y() as usize * self.stride + pos.x() as usize;
        if let Some(elem) = self.grid.to_mut().get_mut(i) {
            *elem = val;
            true
        } else {
            false
        }
    }

    // pub fn rotate_right(&self) -> GridView<'a, T, 2> {
    //     let mut res = self.clone();
    //     res.orientation = res.orientation.rotate_right();
    //     res.size = Point::new(self.size.y, self.size.x);
    //     res
    // }

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
            stride,
            offset: Point::default(),
            size: Point2::new(stride as isize, h as isize),
            orientation: 3,
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

impl<T: Clone + Display> GridView<'_, T, 2> {
    pub fn print(&self) {
        for y in 0..self.size.y() {
            for x in 0..self.size.x() {
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

impl<T: Clone + Display> Debug for GridView<'_, T, 2> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // writeln!(f)?;
        for y in 0..self.size.y() {
            for x in 0..self.size.x() {
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
    i: isize,
}

impl<const N: usize> Iterator for PointIter<N> {
    type Item = Point<N>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= self.size.components.into_iter().product() {
            return None;
        }
        let mut out = [0isize; N];
        let i = self.i;
        self.i += 1;
        self.size
            .components
            .into_iter()
            .enumerate()
            .rev()
            .fold(i, |i, (ndx, max)| {
                out[ndx] = i % max;
                i / max
            });
        Some(Point { components: out })
    }
}
