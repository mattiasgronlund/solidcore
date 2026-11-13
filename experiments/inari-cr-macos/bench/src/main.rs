use inari::{cr, interval, Interval};
use std::hint::black_box;
use std::time::Instant;

struct Rng(u64);
impl Rng { fn next(&mut self) -> f64 { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17; (self.0 >> 11) as f64 / (1u64 << 53) as f64 } }

fn cmp1(name: &str, lo: f64, hi: f64, f: fn(f64) -> f64, g: fn(f64) -> f64) {
    let mut r = Rng(0x9E3779B97F4A7C15);
    let n = 2_000_000;
    let xs: Vec<f64> = (0..n).map(|_| lo + (hi - lo) * r.next()).collect();
    let diff = xs.iter().filter(|&&x| f(x).to_bits() != g(x).to_bits()).count();
    let t = Instant::now(); let mut s = 0.0; for &x in &xs { s += f(black_box(x)); } black_box(s); let tf = t.elapsed().as_nanos() as f64 / n as f64;
    let t = Instant::now(); let mut s = 0.0; for &x in &xs { s += g(black_box(x)); } black_box(s); let tg = t.elapsed().as_nanos() as f64 / n as f64;
    println!("{name:8} [{lo},{hi}] differ {:.4}%  libm {tf:.1} ns  cr {tg:.1} ns  ratio {:.1}x", 100.0 * diff as f64 / n as f64, tg / tf);
}

fn main() {
    cmp1("sin", -7.0, 7.0, f64::sin, cr::sin_rn);
    cmp1("cos", -7.0, 7.0, f64::cos, cr::cos_rn);
    cmp1("sin", -1e4, 1e4, f64::sin, cr::sin_rn);
    cmp1("tan", -1.5, 1.5, f64::tan, cr::tan_rn);
    cmp1("acos", -1.0, 1.0, f64::acos, cr::acos_rn);
    cmp1("asin", -1.0, 1.0, f64::asin, cr::asin_rn);
    cmp1("atan", -10.0, 10.0, f64::atan, cr::atan_rn);
    cmp1("exp", -10.0, 10.0, f64::exp, cr::exp_rn);
    cmp1("ln", 1e-3, 1e3, f64::ln, cr::ln_rn);
    // atan2
    let mut r = Rng(12345); let n = 2_000_000;
    let ps: Vec<(f64,f64)> = (0..n).map(|_| (r.next()*2.0-1.0, r.next()*2.0-1.0)).collect();
    let d = ps.iter().filter(|&&(y,x)| y.atan2(x).to_bits() != cr::atan2_rn(y,x).to_bits()).count();
    let t = Instant::now(); let mut s=0.0; for &(y,x) in &ps { s += black_box(y).atan2(x);} black_box(s); let a = t.elapsed().as_nanos() as f64/n as f64;
    let t = Instant::now(); let mut s=0.0; for &(y,x) in &ps { s += cr::atan2_rn(black_box(y),x);} black_box(s); let b = t.elapsed().as_nanos() as f64/n as f64;
    println!("atan2    [-1,1]^2 differ {:.4}%  libm {a:.1} ns  cr {b:.1} ns  ratio {:.1}x", 100.0*d as f64/n as f64, b/a);
    // interval arithmetic: dot product of 3-vectors, inari vs plain f64 vs a widen-by-2u scheme like Ivl
    let v: Vec<f64> = (0..6*n).map(|_| r.next()*2.0-1.0).collect();
    let t = Instant::now(); let mut s=0.0; for c in v.chunks_exact(6) { let c = black_box(c); s += c[0]*c[3]+c[1]*c[4]+c[2]*c[5]; } black_box(s); let pf = t.elapsed().as_nanos() as f64/n as f64;
    let wid = |x: f64| x + x.abs()*f64::EPSILON; let wdn = |x: f64| x - x.abs()*f64::EPSILON;
    let t = Instant::now(); let mut s=0.0; for c in v.chunks_exact(6) { let c = black_box(c);
        let mut lo=0.0; let mut hi=0.0; for i in 0..3 { let p=c[i]*c[i+3]; lo=wdn(lo+wdn(p)); hi=wid(hi+wid(p)); } s += hi-lo; } black_box(s); let pw = t.elapsed().as_nanos() as f64/n as f64;
    let t = Instant::now(); let mut s=0.0; for c in v.chunks_exact(6) { let c = black_box(c);
        let x: [Interval;6] = core::array::from_fn(|i| interval!(c[i],c[i]).unwrap());
        let d = x[0]*x[3]+x[1]*x[4]+x[2]*x[5]; s += d.wid(); } black_box(s); let pi = t.elapsed().as_nanos() as f64/n as f64;
    println!("dot3     f64 {pf:.1} ns  widen-2u ivl {pw:.1} ns  inari {pi:.1} ns");
    let x = interval!(0.1, 0.1000001).unwrap(); let y = x.sin(); println!("inari sin([0.1,0.1000001]) = {y:?}");
}
