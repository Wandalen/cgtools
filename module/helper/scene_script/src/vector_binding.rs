mod private
{
  use ndarray_cg::{ F32x1, F32x2, F32x3, F32x4, F64x1, F64x2, F64x3, F64x4 };
  use rhai::Engine;

  /// Registers `F32x1` into `engine`: constructor `f32x1( x )`, `.x` property
  /// getter, and `+`/`-`/`*` operators reusing `ndarray_cg`'s own `std::ops`
  /// implementations.
  ///
  /// Registered manually rather than via `#[derive(CustomType)]` because
  /// `F32x1` is foreign to this crate — implementing Rhai's `CustomType`
  /// trait on it here would violate Rust's orphan rule.
  ///
  /// Scalars cross the Rhai boundary as `f64` (Rhai's default `FLOAT`) and
  /// are cast to `f32` at the edge — Rhai's dynamic dispatch matches
  /// registered functions by exact parameter type, so a `f32`-typed
  /// registration never matches a script's `f64` literal. See
  /// [`f64x1_register`] for the `f64`-element sibling, which needs no such
  /// cast; register both to let a script pick either precision.
  ///
  /// Also registers the arity-generic math already implemented on
  /// `ndarray_cg::Vector` — `.dot()`, `.mag()`, `.mag2()`, `.normalize()`
  /// (returns a *new* unit-length copy; does not mutate in place, despite
  /// the name), `.distance()`, `.distance_squared()` (skips the `.mag()`-style
  /// square root; cheaper when only comparing relative distances),
  /// `.min()`, `.max()`, and unary `-` negation. Every scalar result is cast
  /// up to `f64` at the boundary for the same reason the constructor casts
  /// down.
  #[ inline ]
  // Rhai's numeric model is `f64`-only (`FLOAT`); every scalar entering a
  // native `f32` type crosses this narrowing cast at the boundary. Intentional
  // and unavoidable given Rhai's API, not a precision bug.
  pub fn f32x1_register( engine : &mut Engine )
  {
    engine
    .register_type_with_name::< F32x1 >( "F32x1" )
    .register_fn( "f32x1", | x : f64 | F32x1::new( x as f32 ) )
    .register_get( "x", | v : &mut F32x1 | f64::from( v.x() ) )
    .register_fn( "+", | a : F32x1, b : F32x1 | a + b )
    .register_fn( "-", | a : F32x1, b : F32x1 | a - b )
    .register_fn( "-", | a : F32x1 | -a )
    .register_fn( "*", | a : F32x1, s : f64 | a * ( s as f32 ) )
    .register_fn( "*", | s : f64, a : F32x1 | a * ( s as f32 ) )
    .register_fn( "dot", | a : F32x1, b : F32x1 | f64::from( a.dot( &b ) ) )
    .register_fn( "mag", | a : F32x1 | f64::from( a.mag() ) )
    .register_fn( "mag2", | a : F32x1 | f64::from( a.mag2() ) )
    .register_fn( "normalize", | a : F32x1 | a.normalize() )
    .register_fn( "distance", | a : F32x1, b : F32x1 | f64::from( a.distance( &b ) ) )
    .register_fn( "distance_squared", | a : F32x1, b : F32x1 | f64::from( a.distance_squared( &b ) ) )
    .register_fn( "min", | a : F32x1, b : F32x1 | a.min( b ) )
    .register_fn( "max", | a : F32x1, b : F32x1 | a.max( b ) )
    .register_fn( "to_string", | v : &mut F32x1 | format!( "F32x1({})", v.x() ) );
  }

  /// Registers `F32x2` into `engine`: constructor `f32x2( x, y )`, `.x`/`.y`
  /// property getters, and `+`/`-`/`*` operators reusing `ndarray_cg`'s own
  /// `std::ops` implementations.
  ///
  /// Registered manually rather than via `#[derive(CustomType)]` because
  /// `F32x2` is foreign to this crate — implementing Rhai's `CustomType`
  /// trait on it here would violate Rust's orphan rule.
  ///
  /// Scalars cross the Rhai boundary as `f64` (Rhai's default `FLOAT`) and
  /// are cast to `f32` at the edge — Rhai's dynamic dispatch matches
  /// registered functions by exact parameter type, so a `f32`-typed
  /// registration never matches a script's `f64` literal. See
  /// [`f64x2_register`] for the `f64`-element sibling, which needs no such
  /// cast; register both to let a script pick either precision.
  #[ inline ]
  // Rhai's numeric model is `f64`-only (`FLOAT`); every scalar entering a
  // native `f32` type crosses this narrowing cast at the boundary. Intentional
  // and unavoidable given Rhai's API, not a precision bug.
  pub fn f32x2_register( engine : &mut Engine )
  {
    engine
    .register_type_with_name::< F32x2 >( "F32x2" )
    .register_fn( "f32x2", | x : f64, y : f64 | F32x2::new( x as f32, y as f32 ) )
    .register_get( "x", | v : &mut F32x2 | f64::from( v.x() ) )
    .register_get( "y", | v : &mut F32x2 | f64::from( v.y() ) )
    .register_fn( "+", | a : F32x2, b : F32x2 | a + b )
    .register_fn( "-", | a : F32x2, b : F32x2 | a - b )
    .register_fn( "-", | a : F32x2 | -a )
    .register_fn( "*", | a : F32x2, s : f64 | a * ( s as f32 ) )
    .register_fn( "*", | s : f64, a : F32x2 | a * ( s as f32 ) )
    .register_fn( "dot", | a : F32x2, b : F32x2 | f64::from( a.dot( &b ) ) )
    .register_fn( "mag", | a : F32x2 | f64::from( a.mag() ) )
    .register_fn( "mag2", | a : F32x2 | f64::from( a.mag2() ) )
    .register_fn( "normalize", | a : F32x2 | a.normalize() )
    .register_fn( "distance", | a : F32x2, b : F32x2 | f64::from( a.distance( &b ) ) )
    .register_fn( "distance_squared", | a : F32x2, b : F32x2 | f64::from( a.distance_squared( &b ) ) )
    .register_fn( "min", | a : F32x2, b : F32x2 | a.min( b ) )
    .register_fn( "max", | a : F32x2, b : F32x2 | a.max( b ) )
    .register_fn( "to_string", | v : &mut F32x2 | format!( "F32x2({}, {})", v.x(), v.y() ) );
  }

  /// Registers `F32x3` into `engine`: constructor `f32x3( x, y, z )`,
  /// `.x`/`.y`/`.z` property getters, and `+`/`-`/`*` operators reusing
  /// `ndarray_cg`'s own `std::ops` implementations. See [`f32x1_register`]
  /// for the boundary-cast rationale and [`f64x3_register`] for the
  /// `f64`-element sibling.
  #[ inline ]
  // Rhai's numeric model is `f64`-only (`FLOAT`); every scalar entering a
  // native `f32` type crosses this narrowing cast at the boundary. Intentional
  // and unavoidable given Rhai's API, not a precision bug.
  pub fn f32x3_register( engine : &mut Engine )
  {
    engine
    .register_type_with_name::< F32x3 >( "F32x3" )
    .register_fn( "f32x3", | x : f64, y : f64, z : f64 | F32x3::new( x as f32, y as f32, z as f32 ) )
    .register_get( "x", | v : &mut F32x3 | f64::from( v.x() ) )
    .register_get( "y", | v : &mut F32x3 | f64::from( v.y() ) )
    .register_get( "z", | v : &mut F32x3 | f64::from( v.z() ) )
    .register_fn( "+", | a : F32x3, b : F32x3 | a + b )
    .register_fn( "-", | a : F32x3, b : F32x3 | a - b )
    .register_fn( "-", | a : F32x3 | -a )
    .register_fn( "*", | a : F32x3, s : f64 | a * ( s as f32 ) )
    .register_fn( "*", | s : f64, a : F32x3 | a * ( s as f32 ) )
    .register_fn( "dot", | a : F32x3, b : F32x3 | f64::from( a.dot( &b ) ) )
    .register_fn( "mag", | a : F32x3 | f64::from( a.mag() ) )
    .register_fn( "mag2", | a : F32x3 | f64::from( a.mag2() ) )
    .register_fn( "normalize", | a : F32x3 | a.normalize() )
    .register_fn( "distance", | a : F32x3, b : F32x3 | f64::from( a.distance( &b ) ) )
    .register_fn( "distance_squared", | a : F32x3, b : F32x3 | f64::from( a.distance_squared( &b ) ) )
    .register_fn( "min", | a : F32x3, b : F32x3 | a.min( b ) )
    .register_fn( "max", | a : F32x3, b : F32x3 | a.max( b ) )
    .register_fn( "cross", | a : F32x3, b : F32x3 | a.cross( b ) )
    // Appends w = 1.0, producing a homogeneous coordinate for matrix/transform math.
    .register_fn( "to_homogenous", | a : F32x3 | a.to_homogenous() )
    .register_fn( "to_string", | v : &mut F32x3 | format!( "F32x3({}, {}, {})", v.x(), v.y(), v.z() ) );
  }

  /// Registers `F32x4` into `engine`: constructor `f32x4( x, y, z, w )`,
  /// `.x`/`.y`/`.z`/`.w` property getters, and `+`/`-`/`*` operators reusing
  /// `ndarray_cg`'s own `std::ops` implementations. See [`f32x1_register`]
  /// for the boundary-cast rationale and [`f64x4_register`] for the
  /// `f64`-element sibling.
  #[ inline ]
  // Rhai's numeric model is `f64`-only (`FLOAT`); every scalar entering a
  // native `f32` type crosses this narrowing cast at the boundary. Intentional
  // and unavoidable given Rhai's API, not a precision bug.
  pub fn f32x4_register( engine : &mut Engine )
  {
    engine
    .register_type_with_name::< F32x4 >( "F32x4" )
    .register_fn
    (
      "f32x4",
      | x : f64, y : f64, z : f64, w : f64 | F32x4::new( x as f32, y as f32, z as f32, w as f32 )
    )
    // `From<(Vec2, Vec2)>` overload: concatenates `xy`'s two components with
    // `zw`'s two components — `(x, y, z, w)`, not a geometric combination.
    .register_fn( "f32x4", | xy : F32x2, zw : F32x2 | F32x4::from( ( xy, zw ) ) )
    .register_get( "x", | v : &mut F32x4 | f64::from( v.x() ) )
    .register_get( "y", | v : &mut F32x4 | f64::from( v.y() ) )
    .register_get( "z", | v : &mut F32x4 | f64::from( v.z() ) )
    .register_get( "w", | v : &mut F32x4 | f64::from( v.w() ) )
    .register_fn( "+", | a : F32x4, b : F32x4 | a + b )
    .register_fn( "-", | a : F32x4, b : F32x4 | a - b )
    .register_fn( "-", | a : F32x4 | -a )
    .register_fn( "*", | a : F32x4, s : f64 | a * ( s as f32 ) )
    .register_fn( "*", | s : f64, a : F32x4 | a * ( s as f32 ) )
    .register_fn( "dot", | a : F32x4, b : F32x4 | f64::from( a.dot( &b ) ) )
    .register_fn( "mag", | a : F32x4 | f64::from( a.mag() ) )
    .register_fn( "mag2", | a : F32x4 | f64::from( a.mag2() ) )
    .register_fn( "normalize", | a : F32x4 | a.normalize() )
    .register_fn( "distance", | a : F32x4, b : F32x4 | f64::from( a.distance( &b ) ) )
    .register_fn( "distance_squared", | a : F32x4, b : F32x4 | f64::from( a.distance_squared( &b ) ) )
    .register_fn( "min", | a : F32x4, b : F32x4 | a.min( b ) )
    .register_fn( "max", | a : F32x4, b : F32x4 | a.max( b ) )
    .register_fn( "truncate", | a : F32x4 | a.truncate() )
    .register_fn
    (
      "to_string",
      | v : &mut F32x4 | format!( "F32x4({}, {}, {}, {})", v.x(), v.y(), v.z(), v.w() )
    );
  }

  /// Registers `F64x1` into `engine`: constructor `f64x1( x )`, `.x`
  /// property getter, and `+`/`-`/`*` operators reusing `ndarray_cg`'s own
  /// `std::ops` implementations.
  ///
  /// `F64x1`'s element type is `f64`, matching Rhai's native `FLOAT` exactly
  /// — unlike [`f32x1_register`], no boundary cast is needed anywhere here.
  #[ inline ]
  pub fn f64x1_register( engine : &mut Engine )
  {
    engine
    .register_type_with_name::< F64x1 >( "F64x1" )
    .register_fn( "f64x1", F64x1::new )
    .register_get( "x", | v : &mut F64x1 | v.x() )
    .register_fn( "+", | a : F64x1, b : F64x1 | a + b )
    .register_fn( "-", | a : F64x1, b : F64x1 | a - b )
    .register_fn( "-", | a : F64x1 | -a )
    .register_fn( "*", | a : F64x1, s : f64 | a * s )
    .register_fn( "*", | s : f64, a : F64x1 | a * s )
    .register_fn( "dot", | a : F64x1, b : F64x1 | a.dot( &b ) )
    .register_fn( "mag", | a : F64x1 | a.mag() )
    .register_fn( "mag2", | a : F64x1 | a.mag2() )
    .register_fn( "normalize", | a : F64x1 | a.normalize() )
    .register_fn( "distance", | a : F64x1, b : F64x1 | a.distance( &b ) )
    .register_fn( "distance_squared", | a : F64x1, b : F64x1 | a.distance_squared( &b ) )
    .register_fn( "min", | a : F64x1, b : F64x1 | a.min( b ) )
    .register_fn( "max", | a : F64x1, b : F64x1 | a.max( b ) )
    .register_fn( "to_string", | v : &mut F64x1 | format!( "F64x1({})", v.x() ) );
  }

  /// Registers `F64x2` into `engine`: constructor `f64x2( x, y )`, `.x`/`.y`
  /// property getters, and `+`/`-`/`*` operators reusing `ndarray_cg`'s own
  /// `std::ops` implementations.
  ///
  /// `F64x2`'s element type is `f64`, matching Rhai's native `FLOAT` exactly
  /// — unlike [`f32x2_register`], no boundary cast is needed anywhere here.
  /// Registering both types side by side (distinct type names and
  /// constructors, `"F32x2"`/`f32x2` vs `"F64x2"`/`f64x2`) lets a script
  /// pick whichever precision it needs; Rhai resolves `+`/`-`/`*` operator
  /// overloads by each call's actual argument types, so the two coexist
  /// without ambiguity.
  #[ inline ]
  pub fn f64x2_register( engine : &mut Engine )
  {
    engine
    .register_type_with_name::< F64x2 >( "F64x2" )
    .register_fn( "f64x2", F64x2::new )
    .register_get( "x", | v : &mut F64x2 | v.x() )
    .register_get( "y", | v : &mut F64x2 | v.y() )
    .register_fn( "+", | a : F64x2, b : F64x2 | a + b )
    .register_fn( "-", | a : F64x2, b : F64x2 | a - b )
    .register_fn( "-", | a : F64x2 | -a )
    .register_fn( "*", | a : F64x2, s : f64 | a * s )
    .register_fn( "*", | s : f64, a : F64x2 | a * s )
    .register_fn( "dot", | a : F64x2, b : F64x2 | a.dot( &b ) )
    .register_fn( "mag", | a : F64x2 | a.mag() )
    .register_fn( "mag2", | a : F64x2 | a.mag2() )
    .register_fn( "normalize", | a : F64x2 | a.normalize() )
    .register_fn( "distance", | a : F64x2, b : F64x2 | a.distance( &b ) )
    .register_fn( "distance_squared", | a : F64x2, b : F64x2 | a.distance_squared( &b ) )
    .register_fn( "min", | a : F64x2, b : F64x2 | a.min( b ) )
    .register_fn( "max", | a : F64x2, b : F64x2 | a.max( b ) )
    .register_fn( "to_string", | v : &mut F64x2 | format!( "F64x2({}, {})", v.x(), v.y() ) );
  }

  /// Registers `F64x3` into `engine`: constructor `f64x3( x, y, z )`,
  /// `.x`/`.y`/`.z` property getters, and `+`/`-`/`*` operators reusing
  /// `ndarray_cg`'s own `std::ops` implementations. `F64x3`'s element type
  /// is `f64`, matching Rhai's native `FLOAT` exactly — unlike
  /// [`f32x3_register`], no boundary cast is needed anywhere here.
  #[ inline ]
  pub fn f64x3_register( engine : &mut Engine )
  {
    engine
    .register_type_with_name::< F64x3 >( "F64x3" )
    .register_fn( "f64x3", F64x3::new )
    .register_get( "x", | v : &mut F64x3 | v.x() )
    .register_get( "y", | v : &mut F64x3 | v.y() )
    .register_get( "z", | v : &mut F64x3 | v.z() )
    .register_fn( "+", | a : F64x3, b : F64x3 | a + b )
    .register_fn( "-", | a : F64x3, b : F64x3 | a - b )
    .register_fn( "-", | a : F64x3 | -a )
    .register_fn( "*", | a : F64x3, s : f64 | a * s )
    .register_fn( "*", | s : f64, a : F64x3 | a * s )
    .register_fn( "dot", | a : F64x3, b : F64x3 | a.dot( &b ) )
    .register_fn( "mag", | a : F64x3 | a.mag() )
    .register_fn( "mag2", | a : F64x3 | a.mag2() )
    .register_fn( "normalize", | a : F64x3 | a.normalize() )
    .register_fn( "distance", | a : F64x3, b : F64x3 | a.distance( &b ) )
    .register_fn( "distance_squared", | a : F64x3, b : F64x3 | a.distance_squared( &b ) )
    .register_fn( "min", | a : F64x3, b : F64x3 | a.min( b ) )
    .register_fn( "max", | a : F64x3, b : F64x3 | a.max( b ) )
    .register_fn( "cross", | a : F64x3, b : F64x3 | a.cross( b ) )
    // Appends w = 1.0, producing a homogeneous coordinate for matrix/transform math.
    .register_fn( "to_homogenous", | a : F64x3 | a.to_homogenous() )
    .register_fn( "to_string", | v : &mut F64x3 | format!( "F64x3({}, {}, {})", v.x(), v.y(), v.z() ) );
  }

  /// Registers `F64x4` into `engine`: constructor `f64x4( x, y, z, w )`,
  /// `.x`/`.y`/`.z`/`.w` property getters, and `+`/`-`/`*` operators reusing
  /// `ndarray_cg`'s own `std::ops` implementations. `F64x4`'s element type
  /// is `f64`, matching Rhai's native `FLOAT` exactly — unlike
  /// [`f32x4_register`], no boundary cast is needed anywhere here.
  #[ inline ]
  pub fn f64x4_register( engine : &mut Engine )
  {
    engine
    .register_type_with_name::< F64x4 >( "F64x4" )
    .register_fn( "f64x4", F64x4::new )
    // `From<(Vec2, Vec2)>` overload: concatenates `xy`'s two components with
    // `zw`'s two components — `(x, y, z, w)`, not a geometric combination.
    .register_fn( "f64x4", | xy : F64x2, zw : F64x2 | F64x4::from( ( xy, zw ) ) )
    .register_get( "x", | v : &mut F64x4 | v.x() )
    .register_get( "y", | v : &mut F64x4 | v.y() )
    .register_get( "z", | v : &mut F64x4 | v.z() )
    .register_get( "w", | v : &mut F64x4 | v.w() )
    .register_fn( "+", | a : F64x4, b : F64x4 | a + b )
    .register_fn( "-", | a : F64x4, b : F64x4 | a - b )
    .register_fn( "-", | a : F64x4 | -a )
    .register_fn( "*", | a : F64x4, s : f64 | a * s )
    .register_fn( "*", | s : f64, a : F64x4 | a * s )
    .register_fn( "dot", | a : F64x4, b : F64x4 | a.dot( &b ) )
    .register_fn( "mag", | a : F64x4 | a.mag() )
    .register_fn( "mag2", | a : F64x4 | a.mag2() )
    .register_fn( "normalize", | a : F64x4 | a.normalize() )
    .register_fn( "distance", | a : F64x4, b : F64x4 | a.distance( &b ) )
    .register_fn( "distance_squared", | a : F64x4, b : F64x4 | a.distance_squared( &b ) )
    .register_fn( "min", | a : F64x4, b : F64x4 | a.min( b ) )
    .register_fn( "max", | a : F64x4, b : F64x4 | a.max( b ) )
    .register_fn( "truncate", | a : F64x4 | a.truncate() )
    .register_fn
    (
      "to_string",
      | v : &mut F64x4 | format!( "F64x4({}, {}, {}, {})", v.x(), v.y(), v.z(), v.w() )
    );
  }
}

crate::mod_interface!
{
  orphan use
  {
    f32x1_register,
    f32x2_register,
    f32x3_register,
    f32x4_register,
    f64x1_register,
    f64x2_register,
    f64x3_register,
    f64x4_register,
  };
}
