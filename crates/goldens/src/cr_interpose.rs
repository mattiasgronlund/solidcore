//! EXPERIMENT ONLY: route every libm call of this binary to inari's correctly rounded functions.
#![allow(unsafe_code, missing_docs, clippy::pedantic)]
use std::sync::atomic::{AtomicU64, Ordering::Relaxed};
use inari::cr;

const NAMES: [&str; 22] = ["sin","cos","tan","asin","acos","atan","atan2","sinh","cosh","tanh","asinh","acosh","atanh","exp","exp2","log","log2","log10","pow","cbrt","hypot","sincos"];
static N: [AtomicU64; 22] = [const { AtomicU64::new(0) }; 22];
#[inline(always)] fn c(i: usize) { if let Some(n) = N.get(i) { n.fetch_add(1, Relaxed); } }

macro_rules! f1 { ($($i:literal $name:ident $cr:path;)*) => { $(
    #[no_mangle] pub extern "C" fn $name(x: f64) -> f64 { c($i); $cr(x) }
)* } }
f1! { 0 sin cr::sin_rn; 1 cos cr::cos_rn; 2 tan cr::tan_rn; 3 asin cr::asin_rn; 5 atan cr::atan_rn;
      7 sinh cr::sinh_rn; 8 cosh cr::cosh_rn; 9 tanh cr::tanh_rn; 10 asinh cr::asinh_rn; 11 acosh cr::acosh_rn; 12 atanh cr::atanh_rn;
      13 exp cr::exp_rn; 14 exp2 cr::exp2_rn; 15 log cr::ln_rn; 16 log2 cr::log2_rn; 17 log10 cr::log10_rn; 19 cbrt cr::cbrt_rn; }
static SAMPLES: std::sync::Mutex<std::collections::BTreeMap<String, u64>> = std::sync::Mutex::new(std::collections::BTreeMap::new());
#[no_mangle] pub extern "C" fn acos(x: f64) -> f64 {
    let n = N[4].fetch_add(1, Relaxed);
    if n % 100_000 == 0 {
        let bt = std::backtrace::Backtrace::force_capture().to_string();
        let frames: Vec<&str> = bt.lines().map(str::trim).filter(|l| !l.starts_with("at ") && (l.contains("solidcore") || l.contains("goldens::units"))).filter(|l| !l.contains("cr_interpose")).take(8).collect();
        if let Ok(mut m) = SAMPLES.lock() { *m.entry(frames.join(" <- ")).or_insert(0) += 1; }
    }
    cr::acos_rn(x)
}
#[no_mangle] pub extern "C" fn atan2(y: f64, x: f64) -> f64 { c(6); cr::atan2_rn(y, x) }
#[no_mangle] pub extern "C" fn pow(x: f64, y: f64) -> f64 { c(18); cr::pow_rn(x, y) }
#[no_mangle] pub extern "C" fn hypot(x: f64, y: f64) -> f64 { c(20); cr::hypot_rn(x, y) }
#[repr(C)] pub struct SinCos { s: f64, c: f64 }
#[no_mangle] pub extern "C" fn __sincos_stret(x: f64) -> SinCos { c(21); SinCos { s: cr::sin_rn(x), c: cr::cos_rn(x) } }
#[no_mangle] pub extern "C" fn sincos(x: f64, s: *mut f64, co: *mut f64) { c(21); unsafe { *s = cr::sin_rn(x); *co = cr::cos_rn(x); } }

extern "C" fn dump() {
    if let Ok(dir) = std::env::var("CR_COUNTS") {
        let line: Vec<String> = NAMES.iter().zip(N.iter()).map(|(n, v)| format!("{n}={}", v.load(Relaxed))).collect();
        let args: Vec<String> = std::env::args().collect();
        let mut out = format!("ARGS {}\n{}\n", args.join(" "), line.join(" "));
        if let Ok(m) = SAMPLES.lock() { for (k, v) in m.iter() { out.push_str(&format!("SAMPLE {v} {k}\n")); } }
        let _ = std::fs::write(format!("{dir}/{}.txt", std::process::id()), out);
    }
}
extern "C" { fn atexit(cb: extern "C" fn()) -> i32; }
pub fn install() { unsafe { atexit(dump); } }
