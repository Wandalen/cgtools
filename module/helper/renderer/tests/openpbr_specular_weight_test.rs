//! Proves that applying OpenPBR `specular_weight` as a factor on `F0` — what
//! `main.frag` does — *is* the spec's adjusted-IOR construction, not an
//! approximation of it.
//!
//! The spec ( ASWF OpenPBR Surface, specular section ) does not scale the
//! reflectance directly. It adjusts the index of refraction:
//!
//! ```text
//! eps   = sgn( eta - 1 ) * sqrt( xi * F_s ),   F_s = |( 1 - eta ) / ( 1 + eta )|^2
//! eta'  = ( 1 + eps ) / ( 1 - eps )
//! ```
//!
//! and evaluates the dielectric Fresnel with `eta'`. That reads as a different
//! model from multiplying `F0` by `xi`, and the adoption plan carried it as an
//! open fidelity gap for exactly that reason. It is not one. Substituting:
//!
//! ```text
//! 1 - eta' = -2 eps / ( 1 - eps )
//! 1 + eta' =  2     / ( 1 - eps )
//! ```
//!
//! so `( 1 - eta' ) / ( 1 + eta' ) = -eps`, and therefore
//! `F0' = eps^2 = xi * F_s` — exactly, for every `eta` and every `xi`.
//!
//! Since the shader's Fresnel is parameterised by `( F0, F90 = 1 )`, feeding it
//! `xi * F_s` reproduces the adjusted-IOR curve identically. What remains
//! different is Schlick versus the exact dielectric Fresnel at mid angles, which
//! is a universal approximation independent of `specular_weight`.
//!
//! This test exists so the equivalence is not re-litigated, and so nobody
//! "fixes" the shader by adding a redundant adjusted-IOR path.

#![ cfg( not( target_arch = "wasm32" ) ) ]

/// Normal-incidence reflectance of a dielectric interface.
fn f_s( ior : f32 ) -> f32
{
  let r = ( 1.0 - ior ) / ( 1.0 + ior );
  r * r
}

/// `F0` after the spec's adjusted-IOR construction, for weight `xi`.
fn f0_via_adjusted_ior( ior : f32, xi : f32 ) -> f32
{
  let eps = ( xi * f_s( ior ) ).sqrt().copysign( ior - 1.0 );
  let adjusted = ( 1.0 + eps ) / ( 1.0 - eps );
  f_s( adjusted )
}

/// `F0` as the shader computes it: the dielectric reflectance scaled by weight.
fn f0_via_factor( ior : f32, xi : f32 ) -> f32
{
  f_s( ior ) * xi
}

#[ test ]
fn specular_weight_as_an_f0_factor_is_the_adjusted_ior()
{
  for ior in [ 1.05_f32, 1.33, 1.4, 1.5, 1.52, 1.8, 2.4, 2.83 ]
  {
    for xi in [ 0.0_f32, 0.1, 0.25, 0.5, 0.75, 1.0 ]
    {
      let adjusted = f0_via_adjusted_ior( ior, xi );
      let factored = f0_via_factor( ior, xi );
      assert!
      (
        ( adjusted - factored ).abs() < 1e-7,
        "ior {ior}, weight {xi}: adjusted-IOR F0 {adjusted} != factored F0 {factored}"
      );
    }
  }
}

/// The two endpoints the plan used to claim agreement at, stated directly: full
/// weight is the untouched interface, zero weight removes the specular lobe.
#[ test ]
fn the_endpoints_are_the_obvious_ones()
{
  for ior in [ 1.33_f32, 1.5, 2.4 ]
  {
    assert!( ( f0_via_adjusted_ior( ior, 1.0 ) - f_s( ior ) ).abs() < 1e-7, "full weight must leave F0 alone" );
    assert!( f0_via_adjusted_ior( ior, 0.0 ).abs() < 1e-9, "zero weight must extinguish F0" );
  }
}

/// Below an IOR of 1 the sign of `eps` flips, which is the only reason the spec
/// carries a `sgn( eta - 1 )` at all. The identity has to survive it.
#[ test ]
fn the_identity_survives_an_ior_below_one()
{
  for ior in [ 0.5_f32, 0.75, 0.9 ]
  {
    for xi in [ 0.25_f32, 0.5, 1.0 ]
    {
      let adjusted = f0_via_adjusted_ior( ior, xi );
      let factored = f0_via_factor( ior, xi );
      assert!
      (
        ( adjusted - factored ).abs() < 1e-7,
        "ior {ior}, weight {xi}: {adjusted} != {factored}"
      );
    }
  }
}
