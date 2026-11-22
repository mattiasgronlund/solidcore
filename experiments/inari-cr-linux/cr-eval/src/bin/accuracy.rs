//! Misrounding rates of glibc (through std), of inari::cr::*_rn and of the `libm` crate against
//! MPFR, on 200,000 random arguments per row; and how often glibc's `sincos` disagrees with its
//! `sin`/`cos`. The `libm` columns come last, so the earlier columns keep their positions.
use cr_eval::{args, rows, F};
use std::hint::black_box;

extern "C" {
    fn sincos(x: f64, s: *mut f64, c: *mut f64);
}

fn pct(k: usize, n: usize) -> String {
    format!("{:.3}%", 100.0 * k as f64 / n as f64)
}

/// How far `a` is from `b`, in units of their bit patterns.
fn ulps(a: f64, b: f64) -> u64 {
    (a.to_bits() as i64 - b.to_bits() as i64).unsigned_abs()
}

fn main() {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(200_000);
    println!(
        "function\trange\tn\tglibc_misrounded\tcr_misrounded\tglibc_vs_cr_differ\tglibc_max_ulps\tlibm_misrounded\tlibm_max_ulps"
    );
    for (i, row) in rows().iter().enumerate() {
        let (xs, ys) = args(row, i, n, 0xACC);
        let (mut bad_g, mut bad_l, mut bad_c, mut diff) = (0, 0, 0, 0);
        let (mut max_g, mut max_l) = (0u64, 0u64);
        for k in 0..n {
            let (g, l, c, r) = match row.f {
                F::One {
                    glibc,
                    libm,
                    cr,
                    mpfr,
                } => {
                    let x = xs[k];
                    (
                        glibc(black_box(x)),
                        libm(black_box(x)),
                        cr(black_box(x)),
                        mpfr(x),
                    )
                }
                F::Two {
                    glibc,
                    libm,
                    cr,
                    mpfr,
                } => {
                    let (x, y) = (xs[k], ys[k]);
                    (
                        glibc(black_box(x), black_box(y)),
                        libm(black_box(x), black_box(y)),
                        cr(black_box(x), black_box(y)),
                        mpfr(x, y),
                    )
                }
            };
            if g.to_bits() != r.to_bits() {
                bad_g += 1;
                max_g = max_g.max(ulps(g, r));
            }
            if l.to_bits() != r.to_bits() {
                bad_l += 1;
                max_l = max_l.max(ulps(l, r));
            }
            if c.to_bits() != r.to_bits() {
                bad_c += 1;
            }
            if g.to_bits() != c.to_bits() {
                diff += 1;
            }
        }
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
            row.name,
            row.range,
            n,
            pct(bad_g, n),
            pct(bad_c, n),
            pct(diff, n),
            max_g,
            pct(bad_l, n),
            max_l
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
