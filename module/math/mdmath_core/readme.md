# mdmath_core

Fundamental functionality and types, representation slices and tuples as vectors.

## Implemented Features

- Vector:
  - Operations:
    - Dot product of two vectors.
    - Magnitude of the vector.
    - Normalizing the vector.
    - Projection on another vector.
    - Angle beetween two vectors.
    - Orthogonal checking between two vectors.
    - Dimension offset of the vector.
  - Mut/Unmut ref from slice and tuple.
  - Mut/Unmut iterators.

## Installation

Add to your example `[dependencies]` in `Cargo.toml` configuration file:
```toml
mdmath_core = { workspace = true }
```

## Examples

### Dot product of two vectors

```rust
  let vec_a = [ 1.0, 2.0, 3.0 ];
  let vec_b = [ 4.0, 5.0, 6.0 ];
  let result = mdmath_core::vector::dot( &vec_a, &vec_b );
  assert_eq!( result, 32.0 );
```

### Magnitude of the vector

```rust
  let vec_a = [ 1.0, 2.0, 3.0 ];
  let result = mdmath_core::vector::mag2( &vec_a );
  assert_ulps_eq!( result, 14.0 );
```

### Normalizing the vector

```rust
  let vec_a = [ 3.0, 4.0 ];
  let mut result = vec_a.clone();
  mdmath_core::vector::normalize( &mut result, &vec_a );
  let expected = [ 0.6, 0.8 ];
  assert_eq!( result, expected );
```

### Projection on another vector

```rust
  let mut vec_a = [ 1.0, 2.0, 3.0 ];
  let vec_b = [ 4.0, 5.0, 6.0 ];
  mdmath_core::vector::project_on( &mut vec_a, &vec_b );
  let expected = [ 1.6623376623376624, 2.077922077922078, 2.4935064935064934 ];
  assert_eq!( vec_a, expected );
```

### Angle beetween two vectors

```rust
  let vec_a = [ 1.0, 0.0 ];
  let vec_b = [ 0.0, 1.0 ];
  let result = mdmath_core::vector::angle( &vec_a, &vec_b );
  assert_ulps_eq!( result, std::f32::consts::FRAC_PI_2 );
```

### Orthogonal checking between two vectors

```rust
  let vec_a = [ 1.0, 0.0 ];
  let vec_b = [ 0.0, 1.0 ];
  assert!( mdmath_core::vector::is_orthogonal( &vec_a, &vec_b ), "Orthogonal test failed for orthogonal vectors" );
```
