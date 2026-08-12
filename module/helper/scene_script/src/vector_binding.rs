mod private
{
  use ndarray_cg::{ F32x2, F64x2 };
  use rhai::Engine;

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
    .register_fn( "*", | a : F32x2, s : f64 | a * ( s as f32 ) )
    .register_fn( "*", | s : f64, a : F32x2 | a * ( s as f32 ) )
    .register_fn( "to_string", | v : &mut F32x2 | format!( "F32x2({}, {})", v.x(), v.y() ) );
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
    .register_fn( "*", | a : F64x2, s : f64 | a * s )
    .register_fn( "*", | s : f64, a : F64x2 | a * s )
    .register_fn( "to_string", | v : &mut F64x2 | format!( "F64x2({}, {})", v.x(), v.y() ) );
  }
}

crate::mod_interface!
{
  orphan use
  {
    f32x2_register,
    f64x2_register,
  };
}
