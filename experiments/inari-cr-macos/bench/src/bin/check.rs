use inari::cr; use rug::{Float, float::Round, ops::*};
struct Rng(u64);
impl Rng { fn next(&mut self) -> f64 { self.0 ^= self.0 << 13; self.0 ^= self.0 >> 7; self.0 ^= self.0 << 17; (self.0 >> 11) as f64 / (1u64 << 53) as f64 } }
fn main() {
    type F = (&'static str, f64, f64, fn(f64)->f64, fn(f64)->f64, fn(&mut Float));
    let fs: [F; 5] = [
        ("sin", -7.0, 7.0, f64::sin, cr::sin_rn, |x| { x.sin_mut(); }),
        ("cos", -7.0, 7.0, f64::cos, cr::cos_rn, |x| { x.cos_mut(); }),
        ("tan", -1.5, 1.5, f64::tan, cr::tan_rn, |x| { x.tan_mut(); }),
        ("acos", -1.0, 1.0, f64::acos, cr::acos_rn, |x| { x.acos_mut(); }),
        ("atan", -10.0, 10.0, f64::atan, cr::atan_rn, |x| { x.atan_mut(); }),
    ];
    for (name, lo, hi, libm, crf, mp) in fs {
        let mut r = Rng(0x9E3779B97F4A7C15); let n = 200_000; let (mut bl, mut bc) = (0, 0);
        for _ in 0..n { let x = lo + (hi-lo)*r.next();
            let mut m = Float::with_val(53, x); m.set_prec_round(53, Round::Nearest); let mut m2 = Float::with_val(200, x); mp(&mut m2);
            let mut t = Float::with_val(53, 0); let _ = t.assign_round(&m2, Round::Nearest); let t = t.to_f64(); let _=m;
            if libm(x).to_bits() != t.to_bits() { bl += 1; } if crf(x).to_bits() != t.to_bits() { bc += 1; } }
        println!("{name:5} vs MPFR: platform libm misrounded {:.3}%  inari cr misrounded {:.4}%", 100.0*bl as f64/n as f64, 100.0*bc as f64/n as f64);
    }
}
