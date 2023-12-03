use std::fmt::Debug;
use std::ops::Neg;
use std::str::FromStr;

pub fn parse_tuple<L: FromStr, R: FromStr>(s: &str, pat: char) -> (L, R)
where
    L::Err: Debug,
    R::Err: Debug,
{
    let (l, r) = s.split_once(pat).unwrap();
    (l.parse().unwrap(), r.parse().unwrap())
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i8)]
pub enum Direction {
    Up = 0,
    Right = 1,
    Down = 2,
    Left = 3,
}
use Direction::*;

use crate::point::Point2;

impl Direction {
    pub const fn new(dir: i8) -> Direction {
        match dir & 0b11 {
            0 => Up,
            1 => Right,
            2 => Down,
            3 => Left,
            _ => unreachable!(),
        }
    }

    pub fn delta(self) -> Point2 {
        let (x, y) = match self {
            Right => (1, 0),
            Down => (0, 1),
            Left => (-1, 0),
            Up => (0, -1),
        };
        Point2::new(x, y)
    }

    pub const fn rotate_right(self) -> Direction {
        Self::new(self as i8 + 1)
    }

    pub const fn rotate_left(self) -> Direction {
        Self::new(self as i8 - 1)
    }
}

impl Neg for Direction {
    type Output = Self;
    fn neg(self) -> Self::Output {
        (self as i8 - 2).into()
    }
}

impl From<i8> for Direction {
    fn from(value: i8) -> Self {
        Self::new(value)
    }
}
