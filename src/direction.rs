use std::array;
use std::ops::{Add, AddAssign, Neg, Sub, SubAssign};

use crate::point::Point;

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Direction<const N: usize>(u8);

impl<const N: usize> Direction<N> {
    pub const fn new(val: u8) -> Self {
        Direction(val % (2 * N) as u8)
    }

    pub fn delta(self) -> Point<N> {
        Point(array::from_fn(|i| {
            if self.0 as usize % N == i {
                if self.0 as usize / N == 0 {
                    1
                } else {
                    -1
                }
            } else {
                0
            }
        }))
    }

    pub fn edge(self, size: Point<N>) -> Point<N> {
        let mut components = [0; N];
        if self.0 >= N as u8 {
            let i = (self.0 as usize) % N;
            components[i] = size.0[i] - 1
        }
        Point(components)
    }
}

impl<const N: usize> Add<isize> for Direction<N> {
    type Output = Self;
    fn add(self, rhs: isize) -> Direction<N> {
        let val = self.0 as isize + N as isize + rhs;
        Direction::new(val as u8)
    }
}

impl<const N: usize> AddAssign<isize> for Direction<N> {
    fn add_assign(&mut self, rhs: isize) {
        let val = self.0 as isize + N as isize + rhs;
        *self = Direction::new(val as u8)
    }
}

impl<const N: usize> Sub<isize> for Direction<N> {
    type Output = Self;
    fn sub(self, rhs: isize) -> Direction<N> {
        let val = self.0 as isize + N as isize - rhs;
        Direction::new(val as u8)
    }
}

impl<const N: usize> SubAssign<isize> for Direction<N> {
    fn sub_assign(&mut self, rhs: isize) {
        let val = self.0 as isize + N as isize - rhs;
        *self = Direction::new(val as u8)
    }
}

impl<const N: usize> Neg for Direction<N> {
    type Output = Self;
    fn neg(self) -> Self {
        Direction::new(self.0 + N as u8)
    }
}

impl Direction<2> {
    pub const EAST: Direction<2> = Direction::new(0);
    pub const SOUTH: Direction<2> = Direction::new(1);
    pub const WEST: Direction<2> = Direction::new(2);
    pub const NORTH: Direction<2> = Direction::new(3);
}
