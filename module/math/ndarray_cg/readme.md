# ndarray_cg

This crate provides math functionality for computer graphics based on `ndarray`. The approach used in ndarray for computer graphics math is highly flexible and performant, even though there are many specialized crates focused on game development and computer graphics.

To use it within workspace, simply add it to the `Cargo.toml`
```toml
ndarray_cg = { workspace = true, features = { "enabled" } }
```

### Example: Trivial

```Rust
use ndarray_cg::*;
// Will create the following matrix
// [ 1.0, 2.0,
//   3.0, 4.0,
//   5.0, 6.0 ]
let matrix = Mat< 2, 2, f32, DescriptorOrderRowMajor >::from_row_major( [ 1.0, 2.0, 3.0, 4.0 ] );

// For computer graphics related matrices, you can use a shortcut:
// Will create the following matrix
// [ 1.0, 2.0,
//   3.0, 4.0 ]
let matrix = Mat2::from_row_major( [ 1.0, 2.0, 3.0, 4.0 ] );

// You can multiply two matrices
let rotated_matrix = mat2x2::rot( 45.0f32.to_radians() ) * matrix;


// You can iterate over your matrix in different ways
let matrix = Mat< 2, 3, f32, DescriptorOrderRowMajor >::from_row_major( [ 1.0, 2.0, 3.0, 4.0, 5.0, 6.0 ] );
for ( id, v ) in rotated_matrix.iter_indexed_msfirst()
{
    println!( " Index: {:?} , value: {:?}", id, v );
}

for ( id, v ) in rotated_matrix.lane_indexed_iter( 1, 0 )
{
    println!( " Index: {:?} , value: {:?}", id, v );
}
```

### Example: Adding

```rust
  use ndarray_cg::*;

  let mat_a = F32x2x2::from_row_major
  ([
    1.0, 2.0,
    3.0, 4.0,
  ]);
  let mat_b = F32x2x2::from_row_major
  ([
    5.0, 6.0,
    7.0, 8.0,
  ]);
  let mut mat_r = F32x2x2::default();

  // Perform addition
  d2::add( &mut mat_r, &mat_a, &mat_b );

  let exp = F32x2x2::from_row_major
  ([
    6.0, 8.0,
    10.0, 12.0,
  ]);

  assert_eq!( mat_r, exp, "Expected {:?}, got {:?}", exp.raw_slice(), mat_r.raw_slice() );

  // Operator overloading
  let mat_r = &mat_a + &mat_b;
  assert_eq!( mat_r, exp, "Expected {:?}, got {:?}", exp.raw_slice(), mat_r.raw_slice() );
```

### Example: Multiplication

```rust
  use ndarray_cg::*;
  use mat::DescriptorOrderRowMajor;

  let mat_a = Mat::< 1, 3, f32, DescriptorOrderRowMajor >::from_row_major
  ([
    1.0, 2.0, 3.0,
  ]);
  let mat_b = Mat::< 3, 2, f32, DescriptorOrderRowMajor >::from_row_major
  ([
    7.0, 8.0,
    9.0, 10.0,
    11.0, 12.0,
  ]);
  let mut mat_r = Mat::< 1, 2, f32, DescriptorOrderRowMajor >::default();

  // Perform multiplication
  d2::mul( &mut mat_r, &mat_a, &mat_b );

  let exp = Mat::< 1, 2, f32, DescriptorOrderRowMajor >::from_row_major
  ([
    58.0, 64.0,
  ]);
  assert_eq!( mat_r, exp, "Expected {:?}, got {:?}", exp.raw_slice(), mat_r.raw_slice() );

  // Operator overloading
  let mat_r = &mat_a * &mat_b;
  assert_eq!( mat_r, exp, "Expected {:?}, got {:?}", exp.raw_slice(), mat_r.raw_slice() );
```

### Example: Angle rotation

```rust
  use ndarray_cg::*;
  use std::f32::consts::PI;

  let angle_radians = PI / 4.0;
  let cos_theta = angle_radians.cos();
  let sin_theta = angle_radians.sin();

  let exp = F32x2x2::from_row_major
  ([
    cos_theta, -sin_theta,
    sin_theta, cos_theta,
  ]);

  let got = mat2x2::rot( angle_radians );

  assert_eq!( got, exp );
```

### Example: Translation

```rust
  use ndarray_cg::*;

  let tx = 2.0;
  let ty = 3.0;

  let exp = F32x3x3::from_row_major
  ([
    1.0, 0.0, tx,
    0.0, 1.0, ty,
    0.0, 0.0, 1.0,
  ]);

  let got = mat2x2h::translate( [ tx, ty ] );

  assert_eq!( got, exp );
```

### Example: Basic scaling

```rust
  use ndarray_cg::*;

  let sx = 2.0;
  let sy = 3.0;

  let exp = F32x2x2::from_row_major
  ([
    sx, 0.0,
    0.0, sy,
  ]);

  let got = mat2x2::scale( [ sx, sy ] );

  assert_eq!( got, exp );
```

### Example: Reflecting

```rust
  use ndarray_cg::*;

  let exp = F32x2x2::from_row_major
  ([
    1.0, 0.0,
    0.0, -1.0,
  ]);

  let got = mat2x2::reflect_x();

  assert_eq!( got, exp );
```

### Example: Shearing

```rust
  use ndarray_cg::*;

  let shx = 1.0;
  let shy = 0.5;

  let exp = F32x2x2::from_row_major
  ([
    1.0, shx,
    shy, 1.0,
  ]);

  let got = mat2x2::shear( [ shx, shy ] );

  assert_eq!( got, exp );
```

### Example: Line iteration

```rust
  use ndarray_cg::*;
  use mat::DescriptorOrderRowMajor;

  let mat = Mat::< 1, 2, f32, DescriptorOrderRowMajor >::from_row_major( [ 1.0, 2.0 ] );
  let row_iter : Vec< _ > = mat.lane_iter( 0, 0 ).collect();
  let exp = vec![ &1.0, &2.0 ];

  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
```
