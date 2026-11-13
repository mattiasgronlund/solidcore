//! Shared parts of the accuracy, speed and interval programs (inari issue #2, Linux run).
pub mod ivl;

use rug::ops::Pow;
use rug::Float;

/// Marsaglia's xorshift64, deterministic inputs.
pub struct Rng(pub u64);
impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    /// Uniform in [a, b).
    pub fn uniform(&mut self, a: f64, b: f64) -> f64 {
        let u = (self.next() >> 11) as f64 / (1u64 << 53) as f64;
        a + (b - a) * u
    }
}

/// (glibc through std, inari::cr::*_rn, MPFR rounded to nearest at 53 bits)
pub enum F {
    One(fn(f64) -> f64, fn(f64) -> f64, fn(f64) -> f64),
    Two(fn(f64, f64) -> f64, fn(f64, f64) -> f64, fn(f64, f64) -> f64),
}

pub struct Row {
    pub name: &'static str,
    pub range: &'static str,
    pub a: f64,
    pub b: f64,
    /// Second argument's range, for two-argument functions.
    pub a2: f64,
    pub b2: f64,
    pub f: F,
}

fn m(x: f64) -> Float {
    Float::with_val(53, x)
}

macro_rules! one {
    ($name:literal, $range:literal, $a:expr, $b:expr, $std:ident, $cr:ident, $mp:ident) => {
        Row {
            name: $name,
            range: $range,
            a: $a,
            b: $b,
            a2: 0.0,
            b2: 0.0,
            f: F::One(|x| x.$std(), inari::cr::$cr, |x| m(x).$mp().to_f64()),
        }
    };
}
macro_rules! two {
    ($name:literal, $range:literal, $a:expr, $b:expr, $a2:expr, $b2:expr, $std:ident, $cr:ident, $mp:ident) => {
        Row {
            name: $name,
            range: $range,
            a: $a,
            b: $b,
            a2: $a2,
            b2: $b2,
            f: F::Two(
                |x, y| x.$std(y),
                inari::cr::$cr,
                |x, y| m(x).$mp(&m(y)).to_f64(),
            ),
        }
    };
}

/// The issue's ten rows first (same ranges), then the other functions the shim routes.
pub fn rows() -> Vec<Row> {
    vec![
        one!("sin", "[-7, 7]", -7.0, 7.0, sin, sin_rn, sin),
        one!("cos", "[-7, 7]", -7.0, 7.0, cos, cos_rn, cos),
        one!("sin", "[-1e4, 1e4]", -1e4, 1e4, sin, sin_rn, sin),
        one!("tan", "[-1.5, 1.5]", -1.5, 1.5, tan, tan_rn, tan),
        one!("acos", "[-1, 1]", -1.0, 1.0, acos, acos_rn, acos),
        one!("asin", "[-1, 1]", -1.0, 1.0, asin, asin_rn, asin),
        one!("atan", "[-10, 10]", -10.0, 10.0, atan, atan_rn, atan),
        // atan2(y, x): y is the receiver in std and the first argument everywhere
        two!("atan2", "[-1, 1]²", -1.0, 1.0, -1.0, 1.0, atan2, atan2_rn, atan2),
        one!("exp", "[-10, 10]", -10.0, 10.0, exp, exp_rn, exp),
        one!("ln", "[1e-3, 1e3]", 1e-3, 1e3, ln, ln_rn, ln),
        // the rest of the shim
        one!("cos", "[-1e4, 1e4]", -1e4, 1e4, cos, cos_rn, cos),
        two!("pow", "[0.01, 100] × [-10, 10]", 0.01, 100.0, -10.0, 10.0, powf, pow_rn, pow),
        two!("hypot", "[-1e3, 1e3]²", -1e3, 1e3, -1e3, 1e3, hypot, hypot_rn, hypot),
        one!("cbrt", "[-1e3, 1e3]", -1e3, 1e3, cbrt, cbrt_rn, cbrt),
        one!("exp2", "[-10, 10]", -10.0, 10.0, exp2, exp2_rn, exp2),
        one!("log2", "[1e-3, 1e3]", 1e-3, 1e3, log2, log2_rn, log2),
        one!("log10", "[1e-3, 1e3]", 1e-3, 1e3, log10, log10_rn, log10),
        one!("sinh", "[-5, 5]", -5.0, 5.0, sinh, sinh_rn, sinh),
        one!("cosh", "[-5, 5]", -5.0, 5.0, cosh, cosh_rn, cosh),
        one!("tanh", "[-5, 5]", -5.0, 5.0, tanh, tanh_rn, tanh),
    ]
}

/// The arguments of row `i`: `n` values x, and for a two-argument row `n` values y.
pub fn args(row: &Row, i: usize, n: usize, salt: u64) -> (Vec<f64>, Vec<f64>) {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15 ^ ((i as u64 + 1) * 0x1000_0001) ^ salt);
    let mut xs = Vec::with_capacity(n);
    let mut ys = Vec::with_capacity(n);
    for _ in 0..n {
        xs.push(rng.uniform(row.a, row.b));
        if matches!(row.f, F::Two(..)) {
            ys.push(rng.uniform(row.a2, row.b2));
        }
    }
    (xs, ys)
}
