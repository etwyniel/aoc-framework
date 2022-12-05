// re-exports
pub use anyhow;
pub use itertools::Itertools;

use anyhow::{bail, Context};

use std::{
    borrow::Cow,
    env::current_exe,
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    sync::Arc,
    time::Duration,
};

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Answer {
    Num(u64),
    Str(Cow<'static, str>),
}

pub use Answer::*;

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

pub trait Day {
    const YEAR: u16;
    const N: u8;
    const EXAMPLE: Option<&'static str> = None;

    type Part1: Part;
    type Part2: Part;

    fn run(session_key: &str) {
        if Self::Part1::N != 0 {
            run_and_display::<Self::Part1>(session_key)
        } else {
            eprintln!(
                "\x1b[1;33mWRN\x1b[0m Day {} part 1 not implemented",
                Self::N
            );
        }
        if Self::Part2::N != 0 {
            run_and_display::<Self::Part2>(session_key)
        } else {
            eprintln!(
                "\x1b[1;33mWRN\x1b[0m Day {} part 2 not implemented",
                Self::N
            );
        }
    }
}

pub trait Part {
    type D: Day;
    const N: u8;
    const EXAMPLE_RESULT: Option<Answer> = None;

    fn run(_input: impl Iterator<Item = String>) -> anyhow::Result<Answer> {
        bail!("Not implemented")
    }

    fn check() -> anyhow::Result<()> {
        let d = Self::D::N;
        let p = Self::N;
        if let (Some(input), Some(expected)) = (Self::D::EXAMPLE, Self::EXAMPLE_RESULT) {
            let result = Self::run(
                BufReader::new(input.trim_matches('\n').as_bytes())
                    .lines()
                    .map(Result::unwrap),
            )
            .context(format!("Invalid example result for part {p} of day {d}"))?;
            if result != expected {
                bail!(
                    "Invalid example result for part {p} of day {d}\n\tGot\t\t{result}\n\tExpected\t{expected}",
                );
            }
        } else {
            println!("No example for part {p} of day {d}");
        }
        Ok(())
    }
}

impl Day for () {
    const YEAR: u16 = 0;
    const N: u8 = 0;
    type Part1 = ();
    type Part2 = ();
}

impl Part for () {
    type D = ();
    const N: u8 = 0;
}

fn day_input_filename<D: Day>() -> String {
    format!("2022-12-{}.in", D::N)
}

pub fn run<P: Part>(session_key: &str) -> anyhow::Result<(Answer, Duration)> {
    let y = P::D::YEAR;
    let d = P::D::N;
    // Check example inputs/outputs
    P::check()?;
    let dir = if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        // if run as `cargo run`, have inputs directory next to src directory
        PathBuf::from(dir)
    } else {
        // otherwise have input directory next to binary
        current_exe()?
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid directory"))?
            .to_owned()
    }
    .join("inputs");
    if !dir.is_dir() {
        std::fs::create_dir(&dir)?;
    }
    let input_file = dir.join(day_input_filename::<P::D>());
    if !input_file.is_file() {
        let url =
            reqwest::Url::parse(&format!("https://adventofcode.com/{y}/day/{d}/input")).unwrap();
        let jar = reqwest::cookie::Jar::default();
        jar.add_cookie_str(&format!("session={session_key}"), &url);
        let client = reqwest::blocking::Client::builder()
            .cookie_provider(Arc::new(jar))
            .build()?;
        let mut resp = client
            .get(url)
            .header(
                reqwest::header::USER_AGENT,
                "aoc-framework by etwyniel@gmail.com",
            )
            .send()?
            .error_for_status()?;
        let mut output = File::create(&input_file)?;
        std::io::copy(&mut resp, &mut output)?;
    }

    let reader = BufReader::new(File::open(input_file)?)
        .lines()
        .map(Result::unwrap);
    let start = std::time::Instant::now();
    let res = P::run(reader)?;
    let delta = start.elapsed();
    Ok((res, delta))
}

pub fn run_and_display<P: Part>(session_key: &str) {
    let y = P::D::YEAR;
    let d = P::D::N;
    let p = P::N;
    match run::<P>(session_key) {
        Ok((res, delta)) => eprintln!(
            "\x1b[1;32mOK \x1b[0m {y}-12-{d:02}.{p} => {res:<10}\t({:.2?})",
            delta
        ),
        Err(err) => eprintln!("\x1b[1;31mERR\x1b[0m {y}-12-{d:02}.{p} => {err:?}"),
    }
}

#[macro_export]
macro_rules! impl_day {
    ($ident:ident: $year:literal[$day:literal]) => {
        impl_day!($ident::{(), ()}: $year[$day], None);
    };
    ($ident:ident::$part1:ty: $year:literal[$day:literal]) => {
        impl_day!($ident::{$part1, ()}: $year[$day], None);
    };
    ($ident:ident::{$part1:ty, $part2:ty}: $year:literal[$day:literal]) => {
        impl_day!($ident::{$part1, $part2}: $year[$day], None);
    };
    ($ident:ident: $year:literal[$day:literal], $example:literal) => {
        impl_day!($ident::{(), ()}: $year[$day], $example);
    };
    ($ident:ident::$part1:ty: $year:literal[$day:literal], $example:literal) => {
        impl_day!($ident::{$part1, ()}: $year[$day], Some($example));
    };
    ($ident:ident::{$part1:ty, $part2:ty}: $year:literal[$day:literal], $example:literal) => {
        impl_day!($ident::{$part1, $part2}: $year[$day], Some($example));
    };
    ($ident:ident::{$part1:ty, $part2:ty}: $year:literal[$day:literal], $example:expr) => {
        impl $crate::Day for $ident {
            const YEAR: u16 = $year;
            const N: u8 = $day;
            const EXAMPLE: Option<&'static str> = $example;
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
        type D = Day1;
        const N: u8 = 1;
        const EXAMPLE_RESULT: Option<Answer> = Some(Num(7));

        fn run(input: impl Iterator<Item = String>) -> anyhow::Result<Answer> {
            Ok(Num(input
                .map(|line| line.parse().unwrap())
                .tuple_windows()
                .map(|(l, r): (u64, u64)| r > l)
                .map(|increased| increased as u64)
                .sum()))
        }
    }

    struct Part2;

    impl Part for Part2 {
        type D = Day1;
        const N: u8 = 2;
        const EXAMPLE_RESULT: Option<Answer> = Some(Num(5));

        fn run(input: impl Iterator<Item = String>) -> anyhow::Result<Answer> {
            Ok(Num(input
                .map(|line| line.parse::<u64>().unwrap())
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
        Part1::check()?;
        Part2::check()?;
        Ok(())
    }
}
