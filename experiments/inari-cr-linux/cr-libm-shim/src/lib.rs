//! Experiment only: defines the C math library's functions that Rust's std (and LLVM's
//! sin/cos fusion into `sincos`) call, as inari's correctly rounded `cr::*_rn`. Linked into
//! a binary, the static linker resolves std's references to these definitions instead of
//! glibc's libm. Same set as the macOS run of inari issue #2, with glibc's `sincos` in place
//! of Apple's `__sincos_stret`. `log1p` and `expm1` are not covered (cr has neither).
//!
//! With the `count` feature, every call is counted, and at exit each process writes its
//! counts to `$CR_COUNT_DIR/<pid>.tsv` (when that variable is set).
#![allow(clippy::missing_safety_doc)]

use inari::cr;

#[cfg(feature = "count")]
mod count {
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering::Relaxed};

    pub const NAMES: [&str; 23] = [
        "sin", "cos", "tan", "asin", "acos", "atan", "sinh", "cosh", "tanh", "asinh", "acosh",
        "atanh", "exp", "exp2", "log", "log2", "log10", "cbrt", "atan2", "pow", "hypot", "sincos",
        "unused",
    ];
    pub static C: [AtomicU64; 23] = [const { AtomicU64::new(0) }; 23];

    #[inline(always)]
    pub fn hit(i: usize) {
        C[i].fetch_add(1, Relaxed);
    }

    extern "C" fn dump() {
        let Some(dir) = std::env::var_os("CR_COUNT_DIR") else {
            return;
        };
        let path = std::path::Path::new(&dir).join(format!("{}.tsv", std::process::id()));
        if let Ok(mut f) = std::fs::File::create(path) {
            for (n, c) in NAMES.iter().zip(&C) {
                let _ = writeln!(f, "{n}\t{}", c.load(Relaxed));
            }
        }
    }

    extern "C" {
        fn atexit(f: extern "C" fn()) -> i32;
    }

    pub fn install() {
        unsafe {
            atexit(dump);
        }
    }
}

macro_rules! hit {
    ($i:expr) => {
        #[cfg(feature = "count")]
        count::hit($i);
    };
}

macro_rules! shim1 {
    ($($i:literal $c:ident => $f:ident),* $(,)?) => {
        $(
            #[no_mangle]
            pub extern "C" fn $c(x: f64) -> f64 {
                hit!($i);
                cr::$f(x)
            }
        )*
        /// Every one-argument shim, as a function pointer: `link` touches them.
        const ONE: &[extern "C" fn(f64) -> f64] = &[$($c),*];
    };
}

shim1!(
    0 sin => sin_rn, 1 cos => cos_rn, 2 tan => tan_rn,
    3 asin => asin_rn, 4 acos => acos_rn, 5 atan => atan_rn,
    6 sinh => sinh_rn, 7 cosh => cosh_rn, 8 tanh => tanh_rn,
    9 asinh => asinh_rn, 10 acosh => acosh_rn, 11 atanh => atanh_rn,
    12 exp => exp_rn, 13 exp2 => exp2_rn,
    14 log => ln_rn, 15 log2 => log2_rn, 16 log10 => log10_rn,
    17 cbrt => cbrt_rn,
);

#[no_mangle]
pub extern "C" fn atan2(y: f64, x: f64) -> f64 {
    hit!(18);
    cr::atan2_rn(y, x)
}

#[no_mangle]
pub extern "C" fn pow(x: f64, y: f64) -> f64 {
    hit!(19);
    cr::pow_rn(x, y)
}

#[no_mangle]
pub extern "C" fn hypot(x: f64, y: f64) -> f64 {
    hit!(20);
    cr::hypot_rn(x, y)
}

/// glibc's `void sincos(double x, double *sin, double *cos)`, which LLVM fuses a `sin` and
/// a `cos` of the same argument into on GNU targets.
#[no_mangle]
pub unsafe extern "C" fn sincos(x: f64, s: *mut f64, c: *mut f64) {
    hit!(21);
    *s = cr::sin_rn(x);
    *c = cr::cos_rn(x);
}

const TWO: &[extern "C" fn(f64, f64) -> f64] = &[atan2, pow, hypot];

/// Call once from `main`: keeps every definition above in the link (and, with `count`,
/// registers the exit-time dump).
pub fn link() {
    for f in ONE {
        std::hint::black_box(*f);
    }
    for f in TWO {
        std::hint::black_box(*f);
    }
    std::hint::black_box(sincos as unsafe extern "C" fn(f64, *mut f64, *mut f64));
    #[cfg(feature = "count")]
    count::install();
}
