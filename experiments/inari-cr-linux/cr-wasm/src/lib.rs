//! cr-eval's speed program without MPFR, for every target: native, wasm32-wasip1 and
//! wasm32-unknown-unknown. Rows, arguments (`Rng`, `args`, the salts) and the timing loop are
//! cr-eval's, so a row here is the same calls as the row of the same name there.
//!
//! Three implementations per row: `std` (whatever std calls: glibc natively, wasi-libc's musl
//! code on wasm32-wasip1, compiler-builtins' copy of the `libm` crate on wasm32-unknown-unknown),
//! `cr` (fushimi's `cr::*_rn`) and `libm` (the `libm` crate, =0.2.16).
//!
//! Reports (each a TSV text):
//! - `speed_report(n, reps)`: ns per call, median of `reps` alternating passes over `n` arguments
//!   (cr-eval's `speed`, salt 0x5EED);
//! - `hash_report(n)`: per row and implementation, an FNV-1a hash of every result's bits on
//!   cr-eval's `accuracy` arguments (salt 0xACC): equal hashes on two targets mean the same bits;
//! - `acos_report(text, n, reps)` and `acos_hash_report(text)`: the same for `acos` on an
//!   acos-args file (cr-eval's `speed --acos-args`).
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::hint::black_box;

/// Marsaglia's xorshift64, deterministic inputs (cr-eval's).
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

pub enum F {
    One {
        std: fn(f64) -> f64,
        libm: fn(f64) -> f64,
        cr: fn(f64) -> f64,
    },
    Two {
        std: fn(f64, f64) -> f64,
        libm: fn(f64, f64) -> f64,
        cr: fn(f64, f64) -> f64,
    },
}

pub struct Row {
    pub name: &'static str,
    pub range: &'static str,
    pub a: f64,
    pub b: f64,
    pub a2: f64,
    pub b2: f64,
    pub f: F,
}

macro_rules! one {
    ($name:literal, $range:literal, $a:expr, $b:expr, $std:ident, $libm:ident, $cr:ident) => {
        Row {
            name: $name,
            range: $range,
            a: $a,
            b: $b,
            a2: 0.0,
            b2: 0.0,
            f: F::One {
                std: |x| x.$std(),
                libm: libm::$libm,
                cr: inari::cr::$cr,
            },
        }
    };
}
macro_rules! two {
    ($name:literal, $range:literal, $a:expr, $b:expr, $a2:expr, $b2:expr, $std:ident, $libm:ident, $cr:ident) => {
        Row {
            name: $name,
            range: $range,
            a: $a,
            b: $b,
            a2: $a2,
            b2: $b2,
            f: F::Two {
                std: |x, y| x.$std(y),
                libm: libm::$libm,
                cr: inari::cr::$cr,
            },
        }
    };
}

/// cr-eval's `rows()`, in the same order (the order seeds each row's arguments).
pub fn rows() -> Vec<Row> {
    vec![
        one!("sin", "[-7, 7]", -7.0, 7.0, sin, sin, sin_rn),
        one!("cos", "[-7, 7]", -7.0, 7.0, cos, cos, cos_rn),
        one!("sin", "[-1e4, 1e4]", -1e4, 1e4, sin, sin, sin_rn),
        one!("tan", "[-1.5, 1.5]", -1.5, 1.5, tan, tan, tan_rn),
        one!("acos", "[-1, 1]", -1.0, 1.0, acos, acos, acos_rn),
        one!("asin", "[-1, 1]", -1.0, 1.0, asin, asin, asin_rn),
        one!("atan", "[-10, 10]", -10.0, 10.0, atan, atan, atan_rn),
        two!("atan2", "[-1, 1]²", -1.0, 1.0, -1.0, 1.0, atan2, atan2, atan2_rn),
        one!("exp", "[-10, 10]", -10.0, 10.0, exp, exp, exp_rn),
        one!("ln", "[1e-3, 1e3]", 1e-3, 1e3, ln, log, ln_rn),
        one!("cos", "[-1e4, 1e4]", -1e4, 1e4, cos, cos, cos_rn),
        two!("pow", "[0.01, 100] × [-10, 10]", 0.01, 100.0, -10.0, 10.0, powf, pow, pow_rn),
        two!("hypot", "[-1e3, 1e3]²", -1e3, 1e3, -1e3, 1e3, hypot, hypot, hypot_rn),
        one!("cbrt", "[-1e3, 1e3]", -1e3, 1e3, cbrt, cbrt, cbrt_rn),
        one!("exp2", "[-10, 10]", -10.0, 10.0, exp2, exp2, exp2_rn),
        one!("log2", "[1e-3, 1e3]", 1e-3, 1e3, log2, log2, log2_rn),
        one!("log10", "[1e-3, 1e3]", 1e-3, 1e3, log10, log10, log10_rn),
        one!("sinh", "[-5, 5]", -5.0, 5.0, sinh, sinh, sinh_rn),
        one!("cosh", "[-5, 5]", -5.0, 5.0, cosh, cosh, cosh_rn),
        one!("tanh", "[-5, 5]", -5.0, 5.0, tanh, tanh, tanh_rn),
    ]
}

/// cr-eval's `args`: the arguments of row `i`.
pub fn args(row: &Row, i: usize, n: usize, salt: u64) -> (Vec<f64>, Vec<f64>) {
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15 ^ ((i as u64 + 1) * 0x1000_0001) ^ salt);
    let mut xs = Vec::with_capacity(n);
    let mut ys = Vec::with_capacity(n);
    for _ in 0..n {
        xs.push(rng.uniform(row.a, row.b));
        if matches!(row.f, F::Two { .. }) {
            ys.push(rng.uniform(row.a2, row.b2));
        }
    }
    (xs, ys)
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
mod clock {
    #[link(wasm_import_module = "env")]
    extern "C" {
        /// `performance.now()`, in milliseconds.
        fn now_ms() -> f64;
    }
    pub fn now_ns() -> f64 {
        // SAFETY: an import without arguments that returns a number.
        unsafe { now_ms() * 1e6 }
    }
}
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
mod clock {
    use std::sync::OnceLock;
    use std::time::Instant;
    static T0: OnceLock<Instant> = OnceLock::new();
    pub fn now_ns() -> f64 {
        T0.get_or_init(Instant::now).elapsed().as_nanos() as f64
    }
}

fn time1(f: fn(f64) -> f64, xs: &[f64]) -> f64 {
    let t = clock::now_ns();
    let mut acc = 0.0;
    for &x in xs {
        acc += f(black_box(x));
    }
    black_box(acc);
    (clock::now_ns() - t) / xs.len() as f64
}

fn time2(f: fn(f64, f64) -> f64, xs: &[f64], ys: &[f64]) -> f64 {
    let t = clock::now_ns();
    let mut acc = 0.0;
    for (&x, &y) in xs.iter().zip(ys) {
        acc += f(black_box(x), black_box(y));
    }
    black_box(acc);
    (clock::now_ns() - t) / xs.len() as f64
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(|a, b| a.total_cmp(b));
    v[v.len() / 2]
}

/// FNV-1a over the bits of each value.
fn fnv(vals: impl Iterator<Item = f64>) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in vals {
        for b in v.to_bits().to_le_bytes() {
            h = (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    h
}

/// cr-eval's `speed` table, with the columns std, cr, libm (and the two ratios).
pub fn speed_report(n: usize, reps: usize) -> String {
    let mut out = String::from("function\trange\tstd_ns\tcr_ns\tratio\tlibm_ns\tcr_vs_libm\n");
    for (i, row) in rows().iter().enumerate() {
        let (xs, ys) = args(row, i, n, 0x5EED);
        let (mut ts, mut tc, mut tl) = (Vec::new(), Vec::new(), Vec::new());
        for _ in 0..reps {
            match row.f {
                F::One { std, libm, cr } => {
                    ts.push(time1(std, &xs));
                    tc.push(time1(cr, &xs));
                    tl.push(time1(libm, &xs));
                }
                F::Two { std, libm, cr } => {
                    ts.push(time2(std, &xs, &ys));
                    tc.push(time2(cr, &xs, &ys));
                    tl.push(time2(libm, &xs, &ys));
                }
            }
        }
        let (s, c, l) = (median(ts), median(tc), median(tl));
        let _ = writeln!(
            out,
            "{}\t{}\t{s:.1}\t{c:.1}\t{:.2}\t{l:.1}\t{:.2}",
            row.name,
            row.range,
            c / s,
            c / l
        );
    }
    out
}

/// Per row, a hash of each implementation's results on cr-eval's `accuracy` arguments, and how
/// many of the `n` results differ from cr's.
pub fn hash_report(n: usize) -> String {
    let mut out =
        String::from("function\trange\tn\tstd_hash\tcr_hash\tlibm_hash\tstd_ne_cr\tlibm_ne_cr\n");
    for (i, row) in rows().iter().enumerate() {
        let (xs, ys) = args(row, i, n, 0xACC);
        let (s, c, l): (Vec<f64>, Vec<f64>, Vec<f64>) = match row.f {
            F::One { std, libm, cr } => (
                xs.iter().map(|&x| std(x)).collect(),
                xs.iter().map(|&x| cr(x)).collect(),
                xs.iter().map(|&x| libm(x)).collect(),
            ),
            F::Two { std, libm, cr } => (
                xs.iter().zip(&ys).map(|(&x, &y)| std(x, y)).collect(),
                xs.iter().zip(&ys).map(|(&x, &y)| cr(x, y)).collect(),
                xs.iter().zip(&ys).map(|(&x, &y)| libm(x, y)).collect(),
            ),
        };
        let ne = |a: &[f64]| a.iter().zip(&c).filter(|(p, q)| p.to_bits() != q.to_bits()).count();
        let _ = writeln!(
            out,
            "{}\t{}\t{n}\t{:016x}\t{:016x}\t{:016x}\t{}\t{}",
            row.name,
            row.range,
            fnv(s.iter().copied()),
            fnv(c.iter().copied()),
            fnv(l.iter().copied()),
            ne(&s),
            ne(&l)
        );
    }
    out
}

/// The arguments of an acos-args text by bucket, each bucket in file order (cr-eval's parser).
pub fn parse_acos_args(text: &str) -> Result<BTreeMap<String, Vec<f64>>, String> {
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
            None if i == 0 => {}
            None => return Err(format!("line {}: expected unit, bucket and 16 hex digits", i + 1)),
        }
    }
    if by_bucket.is_empty() {
        return Err("no arguments".into());
    }
    Ok(by_bucket)
}

fn buckets(by_bucket: &BTreeMap<String, Vec<f64>>) -> Vec<(String, Vec<f64>)> {
    let all: Vec<f64> = by_bucket.values().flatten().copied().collect();
    by_bucket
        .iter()
        .map(|(b, xs)| (b.clone(), xs.clone()))
        .chain([("all".to_string(), all)])
        .collect()
}

const STD_ACOS: fn(f64) -> f64 = |x| x.acos();
const CR_ACOS: fn(f64) -> f64 = inari::cr::acos_rn;
const LIBM_ACOS: fn(f64) -> f64 = libm::acos;

/// cr-eval's `speed --acos-args` table.
pub fn acos_report(text: &str, n: usize, reps: usize) -> Result<String, String> {
    let mut out = String::from("bucket\targs\tstd_ns\tcr_ns\tratio\tlibm_ns\tcr_vs_libm\n");
    for (bucket, xs) in buckets(&parse_acos_args(text)?) {
        let pass: Vec<f64> = xs.iter().copied().cycle().take(n.max(xs.len())).collect();
        let (mut ts, mut tc, mut tl) = (Vec::new(), Vec::new(), Vec::new());
        for _ in 0..reps {
            ts.push(time1(STD_ACOS, &pass));
            tc.push(time1(CR_ACOS, &pass));
            tl.push(time1(LIBM_ACOS, &pass));
        }
        let (s, c, l) = (median(ts), median(tc), median(tl));
        let _ = writeln!(
            out,
            "{bucket}\t{}\t{s:.1}\t{c:.1}\t{:.2}\t{l:.1}\t{:.2}",
            xs.len(),
            c / s,
            c / l
        );
    }
    Ok(out)
}

/// Per bucket, a hash of each implementation's `acos` of every argument.
pub fn acos_hash_report(text: &str) -> Result<String, String> {
    let mut out = String::from("bucket\targs\tstd_hash\tcr_hash\tlibm_hash\n");
    for (bucket, xs) in buckets(&parse_acos_args(text)?) {
        let h = |f: fn(f64) -> f64| fnv(xs.iter().map(|&x| f(x)));
        let _ = writeln!(
            out,
            "{bucket}\t{}\t{:016x}\t{:016x}\t{:016x}",
            xs.len(),
            h(STD_ACOS),
            h(CR_ACOS),
            h(LIBM_ACOS)
        );
    }
    Ok(out)
}

/// The reports by mode number, for `wspeed` and the JavaScript driver: 0 speed, 1 hash,
/// 2 acos speed, 3 acos hash. `input` is the acos-args text for modes 2 and 3.
pub fn run(mode: u32, input: &str, n: usize, reps: usize) -> Result<String, String> {
    match mode {
        0 => Ok(speed_report(n, reps)),
        1 => Ok(hash_report(n)),
        2 => acos_report(input, n, reps),
        3 => acos_hash_report(input),
        _ => Err(format!("unknown mode {mode}")),
    }
}

/// The exports the JavaScript driver calls (wasm32-unknown-unknown has no other way in or out).
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
mod exports {
    use std::sync::Mutex;
    static IN: Mutex<Vec<u8>> = Mutex::new(Vec::new());
    static OUT: Mutex<String> = Mutex::new(String::new());

    /// A buffer of `len` bytes for the input text; the driver writes into it.
    #[no_mangle]
    pub extern "C" fn input_buffer(len: u32) -> *mut u8 {
        let mut b = IN.lock().unwrap();
        *b = vec![0; len as usize];
        b.as_mut_ptr()
    }

    /// Runs a report into the output buffer; 0 on success, 1 with the error there instead.
    #[no_mangle]
    pub extern "C" fn run(mode: u32, n: u32, reps: u32) -> u32 {
        let input = IN.lock().unwrap();
        let text = std::str::from_utf8(&input).unwrap_or("");
        let (code, s) = match super::run(mode, text, n as usize, reps as usize) {
            Ok(s) => (0, s),
            Err(e) => (1, e),
        };
        *OUT.lock().unwrap() = s;
        code
    }

    #[no_mangle]
    pub extern "C" fn output_ptr() -> *const u8 {
        OUT.lock().unwrap().as_ptr()
    }

    #[no_mangle]
    pub extern "C" fn output_len() -> u32 {
        OUT.lock().unwrap().len() as u32
    }
}
