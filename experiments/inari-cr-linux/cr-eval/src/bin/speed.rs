//! ns per call of the platform's libm (through std) and inari::cr::*_rn: independent calls
//! through a function pointer, summed into an accumulator; `reps` timed passes of each,
//! alternating, the median reported.
//!
//! `speed [n] [reps]`: every row of `rows()`, on `n` random arguments (default 2,000,000).
//!
//! `speed --acos-args <file> [n] [reps]`: `acos` and `acos_rn` on the arguments in `file`,
//! one line each, tab-separated: the unit, the call-site bucket, and the argument's bits as
//! 16 hex digits. A first line that does not parse is taken as a header; any other is an error.
//! One row per bucket, in sorted order, then `all`. Each row's arguments are taken in file
//! order and repeated until a pass makes at least `n` calls.
//!
//! The `glibc_ns` column is whatever libm std links: glibc on Linux, Apple's libm on macOS.
use cr_eval::{args, rows, F};
use std::collections::BTreeMap;
use std::hint::black_box;
use std::process::exit;
use std::time::Instant;

fn time1(f: fn(f64) -> f64, xs: &[f64]) -> f64 {
    let t = Instant::now();
    let mut acc = 0.0;
    for &x in xs {
        acc += f(black_box(x));
    }
    black_box(acc);
    t.elapsed().as_secs_f64() * 1e9 / xs.len() as f64
}

fn time2(f: fn(f64, f64) -> f64, xs: &[f64], ys: &[f64]) -> f64 {
    let t = Instant::now();
    let mut acc = 0.0;
    for (&x, &y) in xs.iter().zip(ys) {
        acc += f(black_box(x), black_box(y));
    }
    black_box(acc);
    t.elapsed().as_secs_f64() * 1e9 / xs.len() as f64
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    v[v.len() / 2]
}

fn functions(n: usize, reps: usize) {
    println!("function\trange\tglibc_ns\tcr_ns\tratio");
    for (i, row) in rows().iter().enumerate() {
        let (xs, ys) = args(row, i, n, 0x5EED);
        let (mut tg, mut tc) = (Vec::new(), Vec::new());
        for _ in 0..reps {
            match row.f {
                F::One(g, c, _) => {
                    tg.push(time1(g, &xs));
                    tc.push(time1(c, &xs));
                }
                F::Two(g, c, _) => {
                    tg.push(time2(g, &xs, &ys));
                    tc.push(time2(c, &xs, &ys));
                }
            }
        }
        let (g, c) = (median(tg), median(tc));
        println!(
            "{}\t{}\t{:.1}\t{:.1}\t{:.2}",
            row.name,
            row.range,
            g,
            c,
            c / g
        );
    }
}

/// The arguments of `path` by bucket, each bucket in file order.
fn read_acos_args(path: &str) -> BTreeMap<String, Vec<f64>> {
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("speed: cannot read {path}: {e}");
        exit(2)
    });
    let mut by_bucket: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        let parsed = match cols.as_slice() {
            [_unit, bucket, bits] if bits.len() == 16 => u64::from_str_radix(bits, 16)
                .ok()
                .map(|b| (bucket.to_string(), f64::from_bits(b))),
            _ => None,
        };
        match parsed {
            Some((bucket, x)) => by_bucket.entry(bucket).or_default().push(x),
            None if i == 0 => {} // a header
            None => {
                eprintln!(
                    "speed: {path}:{}: expected unit, bucket and 16 hex digits: {line:?}",
                    i + 1
                );
                exit(2)
            }
        }
    }
    if by_bucket.is_empty() {
        eprintln!("speed: {path} holds no arguments");
        exit(2)
    }
    by_bucket
}

fn acos_args(path: &str, n: usize, reps: usize) {
    let by_bucket = read_acos_args(path);
    let all: Vec<f64> = by_bucket.values().flatten().copied().collect();
    let glibc: fn(f64) -> f64 = |x| x.acos();
    let cr: fn(f64) -> f64 = inari::cr::acos_rn;
    println!("bucket\targs\tglibc_ns\tcr_ns\tratio");
    for (bucket, xs) in by_bucket
        .iter()
        .map(|(b, xs)| (b.as_str(), xs))
        .chain([("all", &all)])
    {
        let pass: Vec<f64> = xs.iter().copied().cycle().take(n.max(xs.len())).collect();
        let (mut tg, mut tc) = (Vec::new(), Vec::new());
        for _ in 0..reps {
            tg.push(time1(glibc, &pass));
            tc.push(time1(cr, &pass));
        }
        let (g, c) = (median(tg), median(tc));
        println!("{bucket}\t{}\t{g:.1}\t{c:.1}\t{:.2}", xs.len(), c / g);
    }
}

fn main() {
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let (acos_file, rest) = match raw.first().map(String::as_str) {
        Some("--acos-args") => match raw.get(1) {
            Some(path) => (Some(path.as_str()), &raw[2..]),
            None => {
                eprintln!("usage: speed [--acos-args <file>] [n] [reps]");
                exit(2)
            }
        },
        _ => (None, &raw[..]),
    };
    let n: usize = rest
        .first()
        .and_then(|s| s.parse().ok())
        .unwrap_or(2_000_000);
    let reps: usize = rest.get(1).and_then(|s| s.parse().ok()).unwrap_or(5);
    match acos_file {
        Some(path) => acos_args(path, n, reps),
        None => functions(n, reps),
    }
}
