//! Misrounding rates of glibc (through std) and of inari::cr::*_rn against MPFR, on 200,000
//! random arguments per row; and how often glibc's `sincos` disagrees with its `sin`/`cos`.
use cr_eval::{args, rows, F};
use std::hint::black_box;

extern "C" {
    fn sincos(x: f64, s: *mut f64, c: *mut f64);
}

fn pct(k: usize, n: usize) -> String {
    format!("{:.3}%", 100.0 * k as f64 / n as f64)
}

fn main() {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(200_000);
    println!("function\trange\tn\tglibc_misrounded\tcr_misrounded\tglibc_vs_cr_differ\tglibc_max_ulps");
    for (i, row) in rows().iter().enumerate() {
        let (xs, ys) = args(row, i, n, 0xACC);
        let (mut bad_g, mut bad_c, mut diff, mut max_ulps) = (0, 0, 0, 0u64);
        for k in 0..n {
            let (g, c, r) = match row.f {
                F::One(g, c, r) => {
                    let x = xs[k];
                    (g(black_box(x)), c(black_box(x)), r(x))
                }
                F::Two(g, c, r) => {
                    let (x, y) = (xs[k], ys[k]);
                    (
                        g(black_box(x), black_box(y)),
                        c(black_box(x), black_box(y)),
                        r(x, y),
                    )
                }
            };
            if g.to_bits() != r.to_bits() {
                bad_g += 1;
                max_ulps = max_ulps.max((g.to_bits() as i64 - r.to_bits() as i64).unsigned_abs());
            }
            if c.to_bits() != r.to_bits() {
                bad_c += 1;
            }
            if g.to_bits() != c.to_bits() {
                diff += 1;
            }
        }
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            row.name,
            row.range,
            n,
            pct(bad_g, n),
            pct(bad_c, n),
            pct(diff, n),
            max_ulps
        );
    }
    // glibc's sincos against its own sin and cos, on [-7, 7]
    let mut rng = cr_eval::Rng(0x51C05);
    let (mut ds, mut dc) = (0, 0);
    for _ in 0..n {
        let x = rng.uniform(-7.0, 7.0);
        let (mut s, mut c) = (0.0, 0.0);
        unsafe { sincos(black_box(x), &mut s, &mut c) };
        if s.to_bits() != black_box(x).sin().to_bits() {
            ds += 1;
        }
        if c.to_bits() != black_box(x).cos().to_bits() {
            dc += 1;
        }
    }
    println!(
        "# glibc sincos vs sin on [-7, 7]: {} differ; vs cos: {} differ (n = {n})",
        pct(ds, n),
        pct(dc, n)
    );
}
