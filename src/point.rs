use std::{
    fmt::{Debug, Display},
    ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Point<const N: usize> {
    pub components: [isize; N],
}

pub type Point2 = Point<2>;
pub type Point3 = Point<3>;

impl<const N: usize> Default for Point<N> {
    fn default() -> Self {
        Point { components: [0; N] }
    }
}

impl<const N: usize> Display for Point<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        for (i, component) in self.components.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{component}")?;
        }
        write!(f, ")")
    }
}

impl<const N: usize> Debug for Point<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self}")
    }
}

impl<const N: usize> Add<Point<N>> for Point<N> {
    type Output = Self;

    fn add(self, rhs: Point<N>) -> Self::Output {
        self.combine(rhs, |l, r| l + r)
    }
}

impl<const N: usize> AddAssign<Point<N>> for Point<N> {
    fn add_assign(&mut self, rhs: Point<N>) {
        self.components
            .iter_mut()
            .zip(rhs.components)
            .for_each(|(l, r)| *l += r)
    }
}

impl<const N: usize> Sub<Point<N>> for Point<N> {
    type Output = Self;

    fn sub(self, rhs: Point<N>) -> Self::Output {
        self.combine(rhs, |l, r| l - r)
    }
}

impl<const N: usize> SubAssign<Point<N>> for Point<N> {
    fn sub_assign(&mut self, rhs: Point<N>) {
        self.components
            .iter_mut()
            .zip(rhs.components)
            .for_each(|(l, r)| *l -= r)
    }
}

impl<const N: usize> Mul<isize> for Point<N> {
    type Output = Self;

    fn mul(self, rhs: isize) -> Self::Output {
        self.copy_map(|comp| comp * rhs)
    }
}

impl<const N: usize> MulAssign<isize> for Point<N> {
    fn mul_assign(&mut self, rhs: isize) {
        self.components.iter_mut().for_each(|comp| *comp *= rhs)
    }
}

impl<const N: usize> Neg for Point<N> {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.copy_map(|comp| -comp)
    }
}

impl<const N: usize> Point<N> {
    fn copy_map<F: Fn(isize) -> isize>(mut self, f: F) -> Self {
        self.components.iter_mut().for_each(|comp| *comp = f(*comp));
        self
    }

    fn combine(mut self, rhs: Self, f: fn(isize, isize) -> isize) -> Self {
        self.components
            .iter_mut()
            .zip(rhs.components.iter())
            .for_each(|(out, &rhs)| *out = f(*out, rhs));
        self
    }

    pub fn abs(self) -> Self {
        self.copy_map(isize::abs)
    }

    pub fn signum(self) -> Self {
        self.copy_map(isize::signum)
    }

    pub fn dist_manhattan(self, other: Self) -> usize {
        self.components
            .into_iter()
            .zip(other.components)
            .map(|(l, r)| l.abs_diff(r))
            .sum()
    }

    pub fn orientation_delta(o: usize) -> Self {
        assert!(o < N * 2);
        let comp_ndx = o % N;
        let neg = o >= N;
        let mut components = [0; N];
        components[comp_ndx] = if neg { -1 } else { 1 };
        Point { components }
    }
}

impl<const N: usize> From<[isize; N]> for Point<N> {
    fn from(components: [isize; N]) -> Self {
        Point { components }
    }
}

impl Point2 {
    pub fn offset(self, w: usize) -> usize {
        self.x() as usize + self.y() as usize * w
    }

    pub const fn x(self) -> isize {
        self.components[0]
    }

    pub const fn y(self) -> isize {
        self.components[1]
    }

    pub const fn new(x: isize, y: isize) -> Self {
        Point { components: [x, y] }
    }

    pub const fn neighbors_diag(self) -> NeighborDiagIter {
        NeighborDiagIter { p: self, i: 0 }
    }
}

impl Point3 {
    pub const fn x(self) -> isize {
        self.components[0]
    }

    pub const fn y(self) -> isize {
        self.components[1]
    }

    pub const fn z(self) -> isize {
        self.components[2]
    }

    pub const fn new(x: isize, y: isize) -> Self {
        Point {
            components: [x, y, 0],
        }
    }
}

pub struct NeighborDiagIter {
    p: Point2,
    i: i8,
}

impl Iterator for NeighborDiagIter {
    type Item = Point2;
    fn next(&mut self) -> Option<Self::Item> {
        if self.i >= 9 {
            return None;
        }
        if self.i == 4 {
            // skip self
            self.i += 1;
        }
        let i = self.i;
        self.i += 1;
        let y = i / 3;
        let x = i % 3;
        Some(self.p + Point2::new(x as isize - 1, y as isize - 1))
    }
}
