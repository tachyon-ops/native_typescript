#[unsafe(no_mangle)]
pub extern "C" fn test_add_f64(a: f64, b: f64) -> f64 {
    a + b
}

#[unsafe(no_mangle)]
pub extern "C" fn test_negate_bool(a: u8) -> f64 {
    if a == 0 { 1.0 } else { 0.0 }
}
