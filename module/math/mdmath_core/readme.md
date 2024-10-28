# mdmath_core

This crate provides fundamental math functionality and types.

To use it within workspace, simply add it to the `Cargo.toml`
```toml
mdmath_core = { workspace = true, features = { "full" } }
```

### Example
```Rust
use mdmath_core::vector;
use mdmath_core::vector::inner_product::*;

// You can do vector math with anything that implements VectorIterMut/VectorIter
// Will return [ 5.0, 7.0, 9.0 ]
let result = sum( &[ 1.0, 2.0, 3,0], &[ 4.0, 5.0, 6.0 ] );
// Will return 5.0
let result = mag( &( 3.0, 4.0 ) );
// Will return [ 0.0, 0.0, 1.0 ]
let result = cross( &[ 1.0, 0.0, 0.0 ], &[ 0.0, 1.0, 0.0 ] );
// Will return true
let result = is_orthogonal( &( 1.0, 0.0, 0.0 ), &( 0.0, 1.0, 0.0 ) );
```
