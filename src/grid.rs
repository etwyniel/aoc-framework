use std::borrow::Cow;
use std::fmt::{Debug, Display};
use std::iter;
use std::ops::{Deref, DerefMut, Index};

use crate::point::Point;
use crate::point::Point2;

#[derive(Clone)]
pub struct GridView<'a, T: Clone, const N: usize> {
    grid: Cow<'a, [T]>,
    stride: usize,
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

impl<'a, T: Clone, const N: usize> GridView<'a, T, N> {
    pub fn points_iter(&self) -> PointIter<N> {
        PointIter {
            size: self.size,
            cur: Point::default(),
        }
    }

    pub fn data(&self) -> &[T] {
        self.grid.as_ref()
    }

    pub fn data_mut(&mut self) -> &mut [T] {
        self.grid.to_mut()
    }
}

impl<'a, T: Clone> GridView<'a, T, 2> {
    // fn lastx(&self) -> isize {
    //     self.size.components[0] - 1
    // }

    // fn lasty(&self) -> isize {
    //     self.size.components[1] - 1
    // }

    pub fn to_global(&self, p: Point2) -> Point2 {
        // let x = p.components[0];
        // let y = p.components[1];
        self.offset + p
    }

    pub fn to_local(&self, p: Point2) -> Point2 {
        p - self.offset
        // let p = p - self.offset;
        // let x = p.components[0];
        // let y = p.components[1];
        // match self.orientation {
        //     3 => Point2::new(x, y),
        //     0 => Point2::new(y, self.lasty() - x),
        //     1 => Point2::new(self.lastx() - x, self.lasty() - y),
        //     2 => Point2::new(self.lastx() - y, x),
        //     _ => unreachable!(),
        // }
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

    pub fn get(&self, index: Point2) -> Option<&T> {
        if index.components[0] >= self.size.components[0]
            || index.components[1] >= self.size.components[1]
            || index.components[0] < 0
            || index.components[1] < 0
        {
            return None;
        }
        let pos = self.to_global(index);
        let i = pos.components[1] as usize * self.stride + pos.components[0] as usize;
        self.grid.get(i)
    }

    pub fn set(&mut self, index: Point2, val: T) -> bool {
        if index.components[0] >= self.size.components[0]
            || index.components[1] >= self.size.components[1]
            || index.components[0] < 0
            || index.components[1] < 0
        {
            return false;
        }
        let pos = self.to_global(index);
        let i = pos.components[1] as usize * self.stride + pos.components[0] as usize;
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

    pub fn offset_to_point(&self, off: usize) -> Point2 {
        Point2::new((off % self.stride) as isize, (off / self.stride) as isize) - self.offset
    }

    pub fn point_to_offset(&self, pt: Point2) -> usize {
        pt.components[0] as usize + pt.components[1] as usize * self.stride
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
            stride,
            offset: Point::default(),
            size: Point2::new(length as isize, height as isize),
        })
    }
}

impl<T: Clone + Display> GridView<'_, T, 2> {
    pub fn print(&self) {
        for y in 0..self.size.components[1] {
            for x in 0..self.size.components[0] {
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
        for y in 0..self.size.components[1] {
            for x in 0..self.size.components[0] {
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

impl<T: Clone + Display> Debug for GridView<'_, T, 2> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // writeln!(f)?;
        for y in 0..self.size.components[1] {
            for x in 0..self.size.components[0] {
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
}

impl<const N: usize> Iterator for PointIter<N> {
    type Item = Point<N>;

    fn next(&mut self) -> Option<Self::Item> {
        let size = &self.size.components;
        let cur = self.cur;
        if cur.components[0] >= size[0] - 1 {
            return None;
        }
        let next = &mut self.cur.components;
        for (next, size) in next.iter_mut().zip(size.iter()).rev() {
            if *next == size - 1 {
                *next = 0;
            } else {
                *next += 1;
                break;
            }
        }
        Some(cur)
    }
}
