//! ns per call of glibc (through std) and inari::cr::*_rn, on 2,000,000 random arguments per
//! row; `reps` timed passes of each, alternating, the median reported.
use cr_eval::{args, rows, F};
use std::hint::black_box;
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

fn main() {
    let mut a = std::env::args().skip(1);
    let n: usize = a.next().and_then(|s| s.parse().ok()).unwrap_or(2_000_000);
    let reps: usize = a.next().and_then(|s| s.parse().ok()).unwrap_or(5);
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
        println!("{}\t{}\t{:.1}\t{:.1}\t{:.2}", row.name, row.range, g, c, c / g);
    }
}
