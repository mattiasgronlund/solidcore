#[no_mangle] pub extern "C" fn sin(x: f64) -> f64 { inari::cr::sin_rn(x) + 1000.0 }
#[repr(C)] pub struct SinCos { s: f64, c: f64 }
#[no_mangle] pub extern "C" fn __sincos_stret(x: f64) -> SinCos { SinCos { s: 2000.0 + x, c: 3000.0 } }
fn main() { let x: f64 = std::hint::black_box(0.5); println!("{} {:?}", x.sin(), x.sin_cos()); }
