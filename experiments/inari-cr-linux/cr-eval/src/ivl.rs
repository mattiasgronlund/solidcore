//! Copied from solidcore's crates/solidcore/src/blend/fillet_section/interval.rs (main dc5cfd7e7): widening, product, Ivl, Add, Mul.
#![allow(dead_code)]
use core::ops::{Add, Mul};

/// `std::min(a, b)`: `(b < a) ? b : a` (a NaN `a` is kept).
#[inline]
pub(crate) fn cmin(a: f64, b: f64) -> f64 {
    if b < a {
        b
    } else {
        a
    }
}

/// `std::max(a, b)`: `(a < b) ? b : a` (a NaN `a` is kept).
#[inline]
pub(crate) fn cmax(a: f64, b: f64) -> f64 {
    if a < b {
        b
    } else {
        a
    }
}

/// `sm_Dn`: moved down by twice the rounding error of an operation that gave `x`
/// (`|x| 2^-53`, or `2^-1075` below the normal range).
#[inline]
#[must_use]
pub fn round_down(x: f64) -> f64 {
    x - widening(x)
}

/// `sm_Up`: moved up by twice the rounding error of an operation that gave `x`.
#[inline]
#[must_use]
pub fn round_up(x: f64) -> f64 {
    x + widening(x)
}

/// The relative part of [`widening`].
const WIDEN_REL: f64 = 2.3e-16;
/// The absolute part of [`widening`].
const WIDEN_ABS: f64 = 1.0e-300;
/// Below this, `|x| WIDEN_REL` is under the normal range (`9e-293 * 2.3e-16 = 2.07e-308`,
/// less than `f64::MIN_POSITIVE`, 2.2250738585072014e-308).
const WIDEN_TINY: f64 = 9.0e-293;

/// `|x| * 2.3e-16 + 1.0e-300`, bit for bit, the amount [`round_down`] and [`round_up`]
/// move `x` by (not the fork's, which writes the expression; the value is the same).
///
/// For `0 < |x| <` [`WIDEN_TINY`] (every end of an interval that holds 0 is about 1e-300
/// from it) the product `|x| * 2.3e-16` is subnormal, and an Intel core takes a microcode
/// assist for every subnormal result: 28 ns against 1.6 ns for a normal one, which was
/// most of a G^n certificate's time. There it is computed in integers as the floating
/// operations round it: the product rounded once to the subnormal grid (`2^-1074`, ties
/// to even), then the sum with 1e-300 rounded to 1e-300's grid (ties to an even
/// significand). Every other `x` (0, a normal product, infinities, NaN) takes the
/// expression itself.
#[inline]
fn widening(x: f64) -> f64 {
    let a = x.abs();
    if a > 0.0 && a < WIDEN_TINY {
        if let Some(w) = widening_tiny(a) {
            return w;
        }
    }
    a * WIDEN_REL + WIDEN_ABS
}

/// The integer significand and the exponent of a positive finite `x`:
/// `x = m 2^(e - 1075)` (a subnormal `x` has no hidden bit and `e = 1`).
fn significand_exponent(x: f64) -> (u64, u64) {
    let bits = x.to_bits();
    let e = (bits >> 52) & 0x7ff;
    let f = bits & ((1 << 52) - 1);
    if e == 0 {
        (f, 1)
    } else {
        (f | (1 << 52), e)
    }
}

/// `a * b`, bit for bit (not the fork's, which writes the product). Where the product is
/// subnormal, an Intel core takes a microcode assist (41 ns against 2 ns; a G^n
/// certificate meets one wherever an end about 1e-300 from 0 meets a number under about
/// 2e-8): there it is the product of the significands in integers, rounded once to the
/// subnormal grid `2^-1074` (ties to even). Every other product is the floating one.
#[inline]
fn product(a: f64, b: f64) -> f64 {
    let (ba, bb) = (a.to_bits(), b.to_bits());
    let (ea, eb) = ((ba >> 52) & 0x7ff, (bb >> 52) & 0x7ff);
    // |a b| < 2^(ea + eb - 2044): under 2^-1075 (rounds to 0, no assist) for ea + eb <= 969;
    // at least 2^-1022 (normal) for ea + eb >= 1024; subnormal inputs (e = 0) and
    // infinities or NaNs (e = 0x7ff) take the floating product
    if ea == 0 || eb == 0 || ea == 0x7ff || eb == 0x7ff || ea + eb <= 969 || ea + eb >= 1024 {
        return a * b;
    }
    product_subnormal(a, b).unwrap_or(a * b)
}

/// [`product`] in integers, for normal `a`, `b` with `969 < ea + eb < 1024`; `None` where
/// the product is not under the normal range after all.
fn product_subnormal(a: f64, b: f64) -> Option<f64> {
    let (ma, ea) = significand_exponent(a.abs());
    let (mb, eb) = significand_exponent(b.abs());
    // |a b| = ma mb 2^(ea + eb - 2150): in units of 2^-1074, ma mb 2^(ea + eb - 1076)
    let sh = 1076_u64.checked_sub(ea + eb)?;
    if !(1..128).contains(&sh) {
        return None;
    }
    let p = u128::from(ma) * u128::from(mb);
    if (p >> sh) >= 1 << 52 {
        return None; // normal
    }
    let n = u64::try_from(shift_rounded(p, sh, |q| q & 1 == 1)).ok()?;
    let sign = (a.to_bits() ^ b.to_bits()) & (1 << 63);
    Some(f64::from_bits(sign | n))
}

/// `q >> sh` rounded to nearest, a tie to the side `odd` does not call odd.
fn shift_rounded(q: u128, sh: u64, odd: impl Fn(u128) -> bool) -> u128 {
    let lo = q >> sh;
    let rest = q & ((1 << sh) - 1);
    let half = 1 << (sh - 1);
    if rest > half || (rest == half && odd(lo)) {
        lo + 1
    } else {
        lo
    }
}

/// [`widening`] for a positive finite `a` under [`WIDEN_TINY`], in integers; `None` where the
/// exponents leave the cases worked out (never for these constants).
fn widening_tiny(a: f64) -> Option<f64> {
    let (ma, ea) = significand_exponent(a);
    let (mc, ec) = significand_exponent(WIDEN_REL);
    let (mf, ef) = significand_exponent(WIDEN_ABS);
    // a * WIDEN_REL = ma mc 2^(ea + ec - 2150), in units of the subnormal grid 2^-1074:
    // ma mc 2^(ea + ec - 1076), rounded once (ties to even)
    let sh = 1076_u64.checked_sub(ea + ec)?;
    if !(1..128).contains(&sh) {
        return None;
    }
    let n = shift_rounded(u128::from(ma) * u128::from(mc), sh, |q| q & 1 == 1);
    // t = n 2^-1074; WIDEN_ABS = mf 2^(ef - 1075), its grid 2^(ef - 1075): t is n 2^(1 - ef)
    // of those, and the sum mf + that is rounded to an integer (ties to an even
    // significand), staying in WIDEN_ABS's binade
    let sh2 = ef.checked_sub(1)?;
    if !(1..64).contains(&sh2) {
        return None;
    }
    let mf = u128::from(mf);
    if (n >> sh2) + mf >= 1 << 53 {
        return None;
    }
    let k = shift_rounded(n, sh2, |q| (mf + q) & 1 == 1);
    let k = u64::try_from(k).ok()?;
    Some(f64::from_bits(WIDEN_ABS.to_bits() + k))
}
/// `sm_Ivl`: the closed interval `[lo, hi]`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ivl {
    /// The lower end.
    pub lo: f64,
    /// The upper end.
    pub hi: f64,
}
impl Ivl {
    pub const fn new(lo: f64, hi: f64) -> Self {
        Self { lo, hi }
    }
    pub const fn whole() -> Self {
        Self::new(f64::NEG_INFINITY, f64::INFINITY)
    }
    pub fn checked(a: f64, b: f64) -> Self {
        if a.is_nan() || b.is_nan() {
            Self::whole()
        } else {
            Self::new(round_down(a), round_up(b))
        }
    }
}
impl Add for Ivl {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        Self::checked(self.lo + b.lo, self.hi + b.hi)
    }
}
impl Mul for Ivl {
    type Output = Self;
    fn mul(self, b: Self) -> Self {
        let p0 = product(self.lo, b.lo);
        let rest = [
            product(self.lo, b.hi),
            product(self.hi, b.lo),
            product(self.hi, b.hi),
        ];
        let mut lo = p0;
        let mut hi = p0;
        for pi in rest {
            if pi.is_nan() {
                return Self::whole();
            }
            lo = cmin(lo, pi);
            hi = cmax(hi, pi);
        }
        Self::checked(lo, hi)
    }
}
