# ndarray_cg

This crate provides math functionality for computer graphics based on `ndarray`. The approach used in ndarray for computer graphics math is highly flexible and performant, even though there are many specialized crates focused on game development and computer graphics.

To use it within workspace, simply add it to the `Cargo.toml`
```toml
ndarray_cg = { workspace = true, features = { "enabled" } }
```

### Example: Trivial

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

### Example: Adding

```rust
  use ndarray_cg::
  {
    Mat,
    d2,
  };

  let mat_a = Mat::< 2, 2, f32 >::zero().raw_set
  ([
    1.0, 2.0,
    3.0, 4.0,
  ]);
  let mat_b = Mat::< 2, 2, f32 >::zero().raw_set
  ([
    5.0, 6.0,
    7.0, 8.0,
  ]);
  let mut mat_r = Mat::< 2, 2, f32 >::zero();

  // Perform addition
  d2::add( &mut mat_r, &mat_a, &mat_b );

  let exp = Mat::< 2, 2, f32 >::zero().raw_set
  ([
    6.0, 8.0,
    10.0, 12.0,
  ]);
  assert_eq!( mat_r.raw_slice(), exp.raw_slice(), "Expected {:?}, got {:?}", exp.raw_slice(), mat_r.raw_slice() );

  // Operator overloading
  let mat_r = &mat_a + &mat_b;
  assert_eq!( mat_r.raw_slice(), exp.raw_slice(), "Expected {:?}, got {:?}", exp.raw_slice(), mat_r.raw_slice() );
```

### Example: Multiplication

```rust
  use ndarray_cg::
  {
    Mat,
    d2,
  };

  let mat_a = Mat::< 1, 3, f32 >::zero().raw_set
  ([
    1.0, 2.0, 3.0,
  ]);
  let mat_b = Mat::< 3, 2, f32 >::zero().raw_set
  ([
    7.0, 8.0,
    9.0, 10.0,
    11.0, 12.0,
  ]);
  let mut mat_r = Mat::< 1, 2, f32 >::zero();

  // Perform multiplication
  d2::mul( &mut mat_r, &mat_a, &mat_b );

  let exp = Mat::< 1, 2, f32 >::zero().raw_set
  ([
    58.0, 64.0,
  ]);
  assert_eq!( mat_r.raw_slice(), exp.raw_slice(), "Expected {:?}, got {:?}", exp.raw_slice(), mat_r.raw_slice() );

  // Operator overloading
  let mat_r = &mat_a * &mat_b;
  assert_eq!( mat_r.raw_slice(), exp.raw_slice(), "Expected {:?}, got {:?}", exp.raw_slice(), mat_r.raw_slice() );
```

### Example: Angle rotation

```rust
  use ndarray_cg::RawSlice;

  let angle_radians = PI / 4.0;
  let cos_theta = angle_radians.cos();
  let sin_theta = angle_radians.sin();

  let exp : [ f32; 4 ] =
  [
    cos_theta, -sin_theta,
    sin_theta, cos_theta,
  ];

  let got = ndarray_cg::mat2x2::rot( angle_radians );

  assert_eq!( got.raw_slice(), exp );
```

### Example: Translation

```rust
  use ndarray_cg::RawSliceMut;

  let tx = 2.0;
  let ty = 3.0;

  let exp = ndarray_cg::Mat::< 3, 3, _ >::zero().raw_set
  ([
    1.0, 0.0, tx,
    0.0, 1.0, ty,
    0.0, 0.0, 1.0,
  ]);

  let got = ndarray_cg::mat2x2h::translate( [ tx, ty ] );

  assert_eq!( got, exp );
```

### Example: Basic scaling

```rust
  use ndarray_cg::RawSlice;

  let sx = 2.0;
  let sy = 3.0;

  let exp = ndarray_cg::Mat::< 2, 2, _ >::zero().raw_set
  ([
    sx, 0.0,
    0.0, sy,
  ]);

  let got = ndarray_cg::mat2x2::scale( [ sx, sy ] );

  assert_eq!( got, exp );
```

### Example: Reflecting

```rust
  use ndarray_cg::RawSliceMut;

  let exp = ndarray_cg::Mat::< 2, 2, _ >::zero().raw_set
  ([
    1.0, 0.0,
    0.0, -1.0,
  ]);

  let got = ndarray_cg::mat2x2::reflect_x();

  assert_eq!( got, exp );
```

### Example: Shearing

```rust
  use ndarray_cg::RawSlice;

  let shx = 1.0;
  let shy = 0.5;

  let exp = ndarray_cg::Mat::< 2, 2, _ >::zero().raw_set
  ([
    1.0, shx,
    shy, 1.0,
  ]);

  let got = ndarray_cg::mat2x2::shear( [ shx, shy ] );

  assert_eq!( got, exp );
```

### Example: Line iteration

```rust
  use ndarray_cg::
  {
    IndexingRef,
    RawSliceMut,
  };

  let mat = ndarray_cg::Mat::< 1, 2, f32 >::zero().raw_set( [ 1.0, 2.0 ] );
  let row_iter : Vec< _ > = mat.lane_iter( 0, 0 ).collect();
  let exp = vec![ &1.0, &2.0 ];

  assert_eq!( row_iter, exp, "Expected {:?}, got {:?}", exp, row_iter );
```
