// re-exports
pub use anyhow;
pub use itertools::Itertools;

use anyhow::{bail, Context};

use std::{
    env::current_exe,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
    sync::Arc,
};

pub trait Day {
    const YEAR: u16;
    const N: u8;
    const EXAMPLE: Option<&'static str> = None;
}

pub trait Part {
    type D: Day;
    const N: u8;
    const EXAMPLE_RESULT: Option<u64> = None;

    fn run(_input: impl Iterator<Item = String>) -> anyhow::Result<u64> {
        bail!("Not implemented")
    }

    fn check() -> anyhow::Result<()> {
        let d = Self::D::N;
        let p = Self::N;
        if let (Some(input), Some(expected)) = (Self::D::EXAMPLE, Self::EXAMPLE_RESULT) {
            let result = Self::run(
                BufReader::new(input.trim().as_bytes())
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

fn day_input_filename<D: Day>() -> String {
    format!("2022-12-{}.in", D::N)
}

pub fn run<P: Part>(session_key: &str) -> anyhow::Result<u64> {
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

    P::run(
        BufReader::new(File::open(input_file)?)
            .lines()
            .map(Result::unwrap),
    )
}

pub fn run_and_display<P: Part>(session_key: &str) {
    let y = P::D::YEAR;
    let d = P::D::N;
    let p = P::N;
    match run::<P>(session_key) {
        Ok(res) => eprintln!("\x1b[1;32mOK \x1b[0m {y}-12-{d:02}.{p} => {res}"),
        Err(err) => eprintln!("\x1b[1;31mERR\x1b[0m {y}-12-{d:02}.{p} => {err:?}"),
    }
}

#[macro_export]
macro_rules! impl_day {
    ($ident:ident: $year:literal[$day:literal]) => {
        impl $crate::Day for $ident {
            const YEAR: u16 = $year;
            const N: u8 = $day;
        }
    };
    ($ident:ident: $year:literal[$day:literal], $example:literal) => {
        impl $crate::Day for $ident {
            const YEAR: u16 = $year;
            const N: u8 = $day;
            const EXAMPLE: Option<&'static str> = Some($example);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Day1;

    impl_day!(Day1: 2021[1], r"
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
        const EXAMPLE_RESULT: Option<u64> = Some(7);

        fn run(input: impl Iterator<Item = String>) -> anyhow::Result<u64> {
            Ok(input
                .map(|line| line.parse().unwrap())
                .tuple_windows()
                .map(|(l, r): (u64, u64)| r > l)
                .map(|increased| increased as u64)
                .sum())
        }
    }

    struct Part2;

    impl Part for Part2 {
        type D = Day1;
        const N: u8 = 2;
        const EXAMPLE_RESULT: Option<u64> = Some(5);

        fn run(input: impl Iterator<Item = String>) -> anyhow::Result<u64> {
            Ok(input
                .map(|line| line.parse::<u64>().unwrap())
                .tuple_windows()
                .map(|(a, b, c)| a + b + c)
                .tuple_windows()
                .map(|(l, r)| r > l)
                .map(|increased| increased as u64)
                .sum())
        }
    }

    #[test]
    fn test_day1_2021() -> anyhow::Result<()> {
        Part1::check()?;
        Part2::check()?;
        Ok(())
    }
}
