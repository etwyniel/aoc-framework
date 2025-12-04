use std::collections::HashMap;
use std::env::{self, current_exe};
use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind, Read, Seek, Write, stderr, stdin};
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, bail};
use reqwest::header::HeaderMap;

use crate::{Answer, Day, OutputType, Part};

const URL_BASE: &str = "https://adventofcode.com";

fn get_client(session_key: &str) -> anyhow::Result<reqwest::blocking::Client> {
    let jar = reqwest::cookie::Jar::default();
    jar.add_cookie_str(&format!("session={session_key}"), &URL_BASE.parse()?);
    let mut headers = HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        "github.com/etwyniel/aoc-framework by etwyniel@gmail.com".parse()?,
    );
    let client = reqwest::blocking::Client::builder()
        .cookie_provider(Arc::new(jar))
        .default_headers(headers)
        .build()?;
    Ok(client)
}

pub struct Checker {
    inputs_dir: PathBuf,
    client: Option<reqwest::blocking::Client>,
    filters: Vec<i8>,
}

impl Checker {
    pub fn new(session_key: Option<String>, filter: &str) -> anyhow::Result<Self> {
        let inputs_dir = if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
            // if run as `cargo run`, have inputs directory next to src directory
            PathBuf::from(dir)
        } else {
            // otherwise have input directory next to binary
            current_exe().unwrap().parent().unwrap().to_owned()
        }
        .join("inputs");
        if !inputs_dir.is_dir() {
            std::fs::create_dir(&inputs_dir)?;
        }
        let client = match session_key {
            Some(session_key) => Some(get_client(&session_key)?),
            None => {
                eprintln!("Could not find AOC_TOKEN in env");
                None
            }
        };
        let default_filter = if filter.trim().is_empty() { 0 } else { -1 };
        let mut filters = vec![default_filter; 25];
        for flt in filter.split(',') {
            if flt.is_empty() {
                continue;
            }
            let mut it = flt.split('.').map(|s| s.parse::<i8>());

            let (day, part) = match (it.next(), it.next()) {
                (Some(day), Some(part)) => (day?, part?),
                (Some(day), None) => (day?, 0),
                _ => continue,
            };
            filters[day as usize - 1] = part;
        }
        Ok(Checker {
            inputs_dir,
            client: client,
            filters,
        })
    }
    pub fn for_part<D: Day, P: Part>(&self) -> PartChecker<'_> {
        PartChecker {
            c: self,
            y: D::YEAR,
            d: D::N,
            p: P::N,
            runner: |reader| P::run(reader),
            example: D::EXAMPLE,
            example2: D::PART2_EXAMPLE,
            example_result: P::EXAMPLE_RESULT,
            benchmark_runner: |reader| P::bench(reader),
        }
    }

    pub fn run_part<D: Day, P: Part>(&self) -> &Self {
        self.for_part::<D, P>().run_and_display();
        self
    }

    pub fn run<D: Day>(&self) -> &Self {
        self.run_part::<D, D::Part1>().run_part::<D, D::Part2>()
    }
}

pub struct PartChecker<'a> {
    c: &'a Checker,
    y: u16,
    d: u8,
    p: u8,
    runner: fn(&mut dyn BufRead) -> anyhow::Result<Answer>,
    example: Option<&'static str>,
    example2: Option<&'static str>,
    example_result: Option<Answer>,
    benchmark_runner: fn(&mut dyn BufRead) -> Option<Duration>,
}

impl<'a> PartChecker<'a> {
    fn output_file(&self) -> PathBuf {
        self.c
            .inputs_dir
            .join(format!("{}-12-{}.out", self.y, self.d))
    }

    fn input_file(&self) -> PathBuf {
        self.c
            .inputs_dir
            .join(format!("{}-12-{}.in", self.y, self.d))
    }

    fn get_saved_answers(&self) -> anyhow::Result<(Option<String>, Vec<(OutputType, String)>)> {
        let f = BufReader::new(File::open(self.output_file())?);
        let mut correct = None;
        let incorrect = f
            .lines()
            .filter_map(|ln| ln.ok())
            .filter_map(|ln| {
                let ln = ln.strip_prefix(&self.p.to_string())?;
                let (c, answer) = ln.trim_start().split_at_checked(1)?;
                let answer = answer.trim().to_string();
                match c {
                    "=" => {
                        correct = Some(answer);
                        None
                    }
                    ">" => Some((OutputType::TooHigh, answer)),
                    "<" => Some((OutputType::TooLow, answer)),
                    "!" => Some((OutputType::Invalid, answer)),
                    _ => None,
                }
            })
            .collect();
        Ok((correct, incorrect))
    }

    fn save_answer(&self, answer: &str, ty: &OutputType) -> anyhow::Result<()> {
        self.save_answer_for_part(answer, ty, self.p)
    }

    fn save_answer_for_part(&self, answer: &str, ty: &OutputType, part: u8) -> anyhow::Result<()> {
        let c = match ty {
            OutputType::Correct => '=',
            OutputType::TooLow => '<',
            OutputType::TooHigh => '>',
            OutputType::Invalid => '!',
            _ => return Ok(()),
        };
        let mut f = File::options()
            .append(true)
            .create(true)
            .open(self.output_file())?;
        writeln!(&mut f, "{}{c}{answer}", part)?;
        Ok(())
    }

    fn submit_answer(&self, res_str: &str) -> anyhow::Result<OutputType> {
        // retrieve http client to submit answer
        let Some(client) = &self.c.client else {
            // no client (missing token), don't know if answer is correct
            return Ok(OutputType::Unknown);
        };

        let y = self.y;
        let d = self.d;
        let p = self.p;
        let always_check = env::var("AOC_ALWAYS_CHECK")
            .map(|v| v != "0" && v != "false")
            .unwrap_or(false);
        if !always_check {
            // prompt user whether to submit answer
            eprintln!("{y}-12-{d}.{p} => {res_str}\nCheck answer? (yes/no): ",);
            stderr().flush()?;

            // read answer
            let mut line = String::new();
            stdin().read_line(&mut line)?;
            if line.trim().to_lowercase() != "yes" {
                return Ok(OutputType::Unknown);
            }
        }

        // submit answer to adventofcode.com
        let mut form = HashMap::new();
        form.insert("level", self.p.to_string());
        form.insert("answer", res_str.to_string());
        let mut resp = client
            .post(format!("{URL_BASE}/{y}/day/{d}/answer"))
            .form(&form)
            .send()
            .context("failed to submit answer")?;

        // read body to determine if answer was correct
        let mut resp_body = String::new();
        resp.read_to_string(&mut resp_body)?;
        if !resp.status().is_success() {
            bail!(
                "request failed with status {}: {}",
                resp.status(),
                resp_body,
            )
        }
        let ty = if !resp_body.contains("not the right answer") {
            OutputType::Correct
        } else if resp_body.contains("too high") {
            OutputType::TooHigh
        } else if resp_body.contains("too low") {
            OutputType::TooLow
        } else {
            OutputType::Invalid
        };
        Ok(ty)
    }

    fn fetch_submitted_answers(&self) -> anyhow::Result<Vec<String>> {
        let Some(client) = &self.c.client else {
            return Ok(Vec::new());
        };

        let mut resp_body = String::new();
        client
            .get(format!("{URL_BASE}/{}/day/{}", self.y, self.d))
            .send()?
            .read_to_string(&mut resp_body)?;

        // extract answers from html
        let mut answers = Vec::with_capacity(2);
        let mut body = resp_body.as_str();
        while let Some(ndx) = body.find("Your puzzle answer was") {
            body = &body[ndx..];
            // find end of opening <code> tag
            let Some(start) = body.find('>') else {
                break;
            };
            body = &body[(start + 1)..];
            // find start of closing </code> tag
            let Some(end) = body.find('<') else { break };
            let answer = body[..end].trim().to_string();
            // advance slice
            body = &body[end..];

            self.save_answer_for_part(
                &answer,
                &OutputType::Correct,
                (answers.len() + 1).min(2) as u8,
            )?;
            answers.push(answer);
        }
        if answers.is_empty() {
            // no answers found in body, create empty outputs file to avoid fetching repeatedly
            File::create(self.output_file())?;
        }
        Ok(answers)
    }

    pub fn check_answer(&self, res: &Answer) -> anyhow::Result<OutputType> {
        let (correct, incorrect) = match self.get_saved_answers() {
            Err(e)
                if e.downcast_ref::<std::io::Error>()
                    .map(|e| e.kind() == ErrorKind::NotFound)
                    .unwrap_or(false) =>
            {
                // no outputs file found, fetch potential existing answers
                let answers = self.fetch_submitted_answers()?;
                // use fetched answer for this part
                (answers.into_iter().nth(self.p as usize - 1), Vec::new())
            }
            Err(e) => return Result::Err(e),
            Ok(output) => output,
        };

        if res == &Answer::Num(0) || res == &Answer::Str("".into()) {
            return Ok(OutputType::Invalid);
        }

        let res_str = res.to_string();
        if let Some(correct) = &correct
            && &res_str == correct
        {
            return Ok(OutputType::Correct);
        }

        if let Some((ty, _)) = incorrect.into_iter().find(|prev| prev.1 == res_str) {
            // previously attempted incorrect answer
            return Ok(ty);
        }

        if let Some(correct) = correct {
            // result is not one of the previous incorrect answers, and not the correct answer
            return Ok(OutputType::Incorrect(correct));
        }

        let ty = self.submit_answer(&res_str)?;

        // save answer, log potential error but continue
        if let Err(e) = self
            .save_answer(&res_str, &ty)
            .context("failed to save answer {res_str}")
        {
            eprintln!("failed to save answer {res_str}: {e}")
        }
        Ok(ty)
    }

    fn check(&self, input: &str) -> anyhow::Result<()> {
        let Some(expected) = &self.example_result else {
            println!("No example");
            return Ok(());
        };
        let result = (self.runner)(&mut BufReader::new(input.trim_matches('\n').as_bytes()))
            .context("Failed to run on example")?;
        if &result != expected {
            bail!("Incorrect example result\n\tGot     \t{result}\n\tExpected\t{expected}",);
        }
        Ok(())
    }

    fn bench(&self, filename: &Path) -> Duration {
        let mut reader = BufReader::new(File::open(filename).unwrap());
        if let Some(d) = (self.benchmark_runner)(&mut reader) {
            return d;
        }
        let count = 100;
        let start = std::time::Instant::now();
        for _ in 0..count {
            reader.seek(std::io::SeekFrom::Start(0)).unwrap();
            (self.runner)(&mut reader).unwrap();
        }
        let delta = start.elapsed();
        delta / count
    }

    pub fn run(&self) -> anyhow::Result<(Answer, OutputType, Duration)> {
        let y = self.y;
        let d = self.d;
        // Check example inputs/outputs
        let example = (self.p == 2)
            .then_some(self.example2)
            .flatten()
            .or(self.example);
        if let Some(example) = example {
            self.check(example)?;
        }
        let input_file = self.input_file();
        if !input_file.is_file() {
            // fetch input file from adventofcode.com
            let Some(client) = &self.c.client else {
                bail!("Missing AOC_TOKEN environment variable, cannot fetch input");
            };
            let url = reqwest::Url::parse(&format!("{URL_BASE}/{y}/day/{d}/input")).unwrap();
            let mut resp = client
                .get(url)
                .header(
                    reqwest::header::USER_AGENT,
                    "github.com/etwyniel/aoc-framework by etwyniel@gmail.com",
                )
                .send()?
                .error_for_status()?;
            let mut output = File::create(&input_file)?;
            std::io::copy(&mut resp, &mut output)?;
        }

        // run part on input file
        let mut reader = BufReader::new(File::open(&input_file)?);
        let start = std::time::Instant::now();
        let res = (self.runner)(&mut reader)?;
        let mut delta = start.elapsed();

        // check answer, run benchmark if correct and fast
        let ty = self.check_answer(&res)?;
        if ty == OutputType::Correct && delta < Duration::from_millis(1) {
            delta = self.bench(&input_file)
        }
        Ok((res, ty, delta))
    }

    pub fn run_and_display(&self) {
        // check if part is filtered out
        let flt = self.c.filters[self.d as usize - 1];
        if flt != 0 && flt != self.p as i8 {
            return;
        }

        let id = format!("{}-12-{:02}.{}", self.y, self.d, self.p);
        let (res, ty, delta) = match self.run() {
            Ok(res) => res,
            Err(err) => {
                eprintln!("\x1b[1;31mERR\x1b[0m {id} => {err:?}");
                return;
            }
        };
        let mut status = "OK";
        let mut color = 32;
        let mut msg = format!("{res:<15}");
        match ty {
            OutputType::Correct => (),
            OutputType::Incorrect(correct) => {
                status = "ERR";
                color = 31;
                msg = format!("invalid result:\n\tGot:      {res}\n\tExpected: {correct}");
            }
            OutputType::TooHigh | OutputType::TooLow | OutputType::Invalid => {
                status = "ERR";
                color = 31;
                let extra = if ty == OutputType::TooHigh {
                    " (too high)"
                } else if ty == OutputType::TooLow {
                    " (too low)"
                } else {
                    ""
                };
                msg = format!("{res:<15}\n\tinvalid result{extra}");
            }
            OutputType::Unknown => {
                status = "UNK";
                color = 33;
            }
        }
        eprintln!("\x1b[1;{color}m{status:<3}\x1b[0m {id} =( {delta:^5.0?} )=> {msg}",)
    }
}
