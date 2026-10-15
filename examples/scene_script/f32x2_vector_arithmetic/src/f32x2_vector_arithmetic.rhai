// Same F32x2 type registered from Rust (ndarray_cg::F32x2) — real +/*
// operators via Rust's own std::ops, not new Rhai-side arithmetic.

let a = f32x2( 10.0, 20.0 );
let b = f32x2( 3.0, -3.0 );

a + b * 2.0
