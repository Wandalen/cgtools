# ndarray_cg

This crate provides math functionality for computer graphics based on `ndarray`. The approach used in ndarray for computer graphics math is highly flexible and performant, even though there are many specialized crates focused on game development and computer graphics.

To use it within workspace, simply add it to the `Cargo.toml`
```toml
ndarray_cg = { workspace = true, features = { "enabled" } }
```

### Example
```Rust
use ndarray_cg::d2;
// Will create the following matrix
// [ 1.0, 2.0,
//   3.0, 4.0,
//   5.0, 6.0 ]
let matrix = mat::Mat< 2, 2, f32, DescriptorOrderRowMajor >::from_row_major( [ 1.0, 2.0, 3.0, 4.0 ] );

// For computer graphics related matrices, you can use a shortcut:
// Will create the following matrix
// [ 1.0, 2.0,
//   3.0, 4.0 ]
let matrix = mat::Mat2::from_row_major( [ 1.0, 2.0, 3.0, 4.0 ] );

// You can multiply two matrices 
let rotated_matrix = mat::mat2x2::rot( 45.0f32.to_radians() ) * matrix;


// You can iterate over your matrix in different ways
let matrix = mat::Mat< 2, 3, f32, DescriptorOrderRowMajor >::from_row_major( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0 ] );
for ( id, v ) in rotated_matrix.iter_indexed_msfirst()
{
    println!( " Index: {:?} , value: {:?}", id, v );
}

for ( id, v ) in rotated_matrix.lane_indexed_iter( 1, 0 )
{
    println!( " Index: {:?} , value: {:?}", id, v );
}
```