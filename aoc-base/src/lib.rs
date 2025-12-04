// re-exports
pub use anyhow;
pub use itertools::Itertools;

pub mod checker;

use anyhow::{Context, bail};

use std::{
    borrow::Cow,
    fmt::Display,
    io::{BufRead, BufReader},
    time::Duration,
};

#[derive(Eq, Clone, Debug)]
pub enum Answer {
    Num(u64),
    Str(Cow<'static, str>),
}

pub use Answer::*;

use crate::checker::Checker;

impl PartialEq for Answer {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Num(l), Num(r)) => l == r,
            (Str(l), Str(r)) => l.trim() == r.trim(),
            _ => false,
        }
    }
}

#[allow(non_snake_case)]
pub const fn ConstStr(s: &'static str) -> Answer {
    Str(Cow::Borrowed(s))
}

impl Display for Answer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Num(n) => n.fmt(f),
            Str(s) => s.fmt(f),
        }
    }
}

impl From<u64> for Answer {
    fn from(value: u64) -> Self {
        Num(value)
    }
}

impl From<String> for Answer {
    fn from(value: String) -> Self {
        Str(Cow::Owned(value))
    }
}

pub trait Day: Sized {
    const YEAR: u16;
    const N: u8;
    const EXAMPLE: Option<&'static str> = None;
    const PART2_EXAMPLE: Option<&'static str> = None;

    type Part1: Part;
    type Part2: Part;

    fn run(session_key: Option<&str>) {
        let checker = Checker::new(session_key.map(str::to_owned), "").unwrap();
        if Self::Part1::N != 0 {
            checker.run_part::<Self, Self::Part1>();
        } else {
            eprintln!(
                "\x1b[1;33mWRN\x1b[0m Day {} part 1 not implemented",
                Self::N
            );
        }
        if Self::Part2::N != 0 {
            checker.run_part::<Self, Self::Part2>();
        } else {
            eprintln!(
                "\x1b[1;33mWRN\x1b[0m Day {} part 2 not implemented",
                Self::N
            );
        }
    }
}

pub trait Part {
    const N: u8;
    const EXAMPLE_RESULT: Option<Answer> = None;

    fn run(_input: impl BufRead) -> anyhow::Result<Answer> {
        bail!("Not implemented")
    }

    fn check(input: &str) -> anyhow::Result<()> {
        let Some(expected) = Self::EXAMPLE_RESULT else {
            println!("No example");
            return Ok(());
        };
        let result = Self::run(BufReader::new(input.trim_matches('\n').as_bytes()))
            .context("Failed to run on example")?;
        if result != expected {
            bail!("Incorrect example result\n\tGot     \t{result}\n\tExpected\t{expected}",);
        }
        Ok(())
    }

    fn bench(_input: impl BufRead) -> Option<Duration> {
        None
    }
}

impl Day for () {
    const YEAR: u16 = 0;
    const N: u8 = 0;
    type Part1 = ();
    type Part2 = ();
}

impl Part for () {
    const N: u8 = 0;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OutputType {
    TooLow,
    TooHigh,
    Incorrect(String),
    Correct,
    Unknown,
    Invalid,
}

#[macro_export]
macro_rules! impl_day {
    ($ident:ident: $year:literal[$day:literal]) => {
        impl_day!($ident::{(), ()}: $year[$day], None, None);
    };
    ($ident:ident::$part1:ty: $year:literal[$day:literal]) => {
        impl_day!($ident::{$part1, ()}: $year[$day], None, None);
    };
    ($ident:ident::{$part1:ty, $part2:ty}: $year:literal[$day:literal]) => {
        impl_day!($ident::{$part1, $part2}: $year[$day], None, None);
    };
    ($ident:ident: $year:literal[$day:literal], $example:literal) => {
        impl_day!($ident::{(), ()}: $year[$day], $example);
    };
    ($ident:ident::$part1:ty: $year:literal[$day:literal], $example:literal) => {
        impl_day!($ident::{$part1, ()}: $year[$day], Some($example), None);
    };
    ($ident:ident::$part1:ty: $year:literal[$day:literal], $example:literal, $example2:literal) => {
        impl_day!($ident::{$part1, ()}: $year[$day], Some($example), Some($example2));
    };
    ($ident:ident::{$part1:ty, $part2:ty}: $year:literal[$day:literal], $example:literal) => {
        impl_day!($ident::{$part1, $part2}: $year[$day], Some($example), None);
    };
    ($ident:ident::{$part1:ty, $part2:ty}: $year:literal[$day:literal], $example:literal, $example2:literal) => {
        impl_day!($ident::{$part1, $part2}: $year[$day], Some($example), Some($example2));
    };
    ($ident:ident::{$part1:ty, $part2:ty}: $year:literal[$day:literal], $example:expr, $example2:expr) => {
        impl $crate::Day for $ident {
            const YEAR: u16 = $year;
            const N: u8 = $day;
            const EXAMPLE: Option<&'static str> = $example;
            const PART2_EXAMPLE: Option<&'static str> = $example2;
            type Part1 = $part1;
            type Part2 = $part2;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Day1;

    impl_day!(Day1::{Part1, Part2}: 2021[1], r"
199
200
208
210
200
207
240
269
260
263
");

    struct Part1;

    impl Part for Part1 {
        const N: u8 = 1;
        const EXAMPLE_RESULT: Option<Answer> = Some(Num(7));

        fn run(input: impl BufRead) -> anyhow::Result<Answer> {
            Ok(Num(input
                .lines()
                .map(|line| line.unwrap().parse().unwrap())
                .tuple_windows()
                .map(|(l, r): (u64, u64)| r > l)
                .map(|increased| increased as u64)
                .sum()))
        }
    }

    struct Part2;

    impl Part for Part2 {
        const N: u8 = 2;
        const EXAMPLE_RESULT: Option<Answer> = Some(Num(5));

        fn run(input: impl BufRead) -> anyhow::Result<Answer> {
            Ok(Num(input
                .lines()
                .map(|line| line.unwrap().parse::<u64>().unwrap())
                .tuple_windows()
                .map(|(a, b, c)| a + b + c)
                .tuple_windows()
                .map(|(l, r)| r > l)
                .map(|increased| increased as u64)
                .sum()))
        }
    }

    #[test]
    fn test_day1_2021() -> anyhow::Result<()> {
        Part1::check(Day1::EXAMPLE.unwrap())?;
        Part2::check(Day1::EXAMPLE.unwrap())?;
        Ok(())
    }
}
