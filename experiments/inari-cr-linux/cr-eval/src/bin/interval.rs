//! A 3-vector dot product of point intervals, building the intervals included: plain f64,
//! solidcore's widened round-to-nearest `Ivl`, and inari::Interval with the build's backend.
//! Best of `reps` passes, each 1000 sweeps over 1024 random vector pairs.
use cr_eval::ivl::Ivl;
use cr_eval::Rng;
use inari::Interval;
use std::hint::black_box;
use std::time::Instant;

const N: usize = 1024;

fn bench(reps: usize, data: &[[f64; 6]], f: impl Fn(&[f64; 6]) -> f64) -> f64 {
    let mut best = f64::INFINITY;
    for _ in 0..reps {
        let t = Instant::now();
        let mut acc = 0.0;
        for _ in 0..1000 {
            for v in data {
                acc += f(black_box(v));
            }
        }
        black_box(acc);
        best = best.min(t.elapsed().as_secs_f64() * 1e9 / (1000 * data.len()) as f64);
    }
    best
}

fn main() {
    let reps: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(7);
    let mut rng = Rng(0xD07);
    let data: Vec<[f64; 6]> = (0..N)
        .map(|_| core::array::from_fn(|_| rng.uniform(-10.0, 10.0)))
        .collect();
    let f = bench(reps, &data, |v| v[0] * v[3] + v[1] * v[4] + v[2] * v[5]);
    let w = bench(reps, &data, |v| {
        let p = |x: f64| Ivl::new(x, x);
        let d = p(v[0]) * p(v[3]) + p(v[1]) * p(v[4]) + p(v[2]) * p(v[5]);
        d.lo + d.hi
    });
    let i = bench(reps, &data, |v| {
        let p = |x: f64| Interval::try_from((x, x)).unwrap();
        let d = p(v[0]) * p(v[3]) + p(v[1]) * p(v[4]) + p(v[2]) * p(v[5]);
        d.inf() + d.sup()
    });
    println!("plain f64\t{f:.1} ns");
    println!("widened RN (solidcore Ivl)\t{w:.1} ns");
    println!("inari::Interval\t{i:.1} ns");
}
