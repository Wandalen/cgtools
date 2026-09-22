//! Verifies the Kulla–Conty multi-scatter energy-compensation table generator
//! ( `renderer::webgl::loaders::kulla_conty::kulla_conty_tables` ) — the pure,
//! off-GPU computation of the GGX directional albedo `E(μ, α)` and hemisphere
//! average `E_avg(α)`, later uploaded as a LUT for direct-light energy
//! compensation. Pure math, natively testable.

use renderer::webgl::loaders::kulla_conty::{ KullaContyTables, effective_roughness, kulla_conty_tables, multi_scatter_fresnel };

/// Cheap-but-accurate table for tests.
fn tables() -> KullaContyTables
{
  kulla_conty_tables( 32, 16, 512 )
}

#[ test ]
fn table_shapes_are_consistent()
{
  let t = tables();
  assert_eq!( t.uv_samples, 32 );
  assert_eq!( t.roughness_samples, 16 );
  assert_eq!( t.e_uv.len(), 32 * 16 );
  assert_eq!( t.e_avg.len(), 16 );
}

/// E(μ, α) is an albedo: bounded by [0, 1].
#[ test ]
fn directional_albedo_is_bounded()
{
  let t = tables();
  for &e in &t.e_uv
  {
    assert!( e.is_finite(), "E must be finite" );
    assert!( e >= 0.0 && e <= 1.0001, "E(μ, α) must be in [0, 1]; got {e}" );
  }
  for &e in &t.e_avg
  {
    assert!( e >= 0.0 && e <= 1.0001, "E_avg must be in [0, 1]; got {e}" );
  }
}

/// A perfectly smooth surface reflects almost everything: E_avg(α→0) ≈ 1.
#[ test ]
fn smooth_surface_conserves_energy()
{
  let t = tables();
  assert!( t.e_avg[ 0 ] > 0.9, "E_avg at roughness 0 should be ~1; got {}", t.e_avg[ 0 ] );
}

/// Roughness loses single-scatter energy: E_avg must be non-increasing in α.
#[ test ]
fn avg_albedo_decreases_with_roughness()
{
  let t = tables();
  for w in t.e_avg.windows( 2 )
  {
    assert!( w[ 1 ] <= w[ 0 ] + 1e-3, "E_avg must be non-increasing in roughness; got {} then {}", w[ 0 ], w[ 1 ] );
  }
}

/// A perfect mirror loses nothing: whatever the microfacets bounce around
/// eventually leaves, so the compensation term returns all of it regardless of
/// how much single-scatter energy the surface lost.
#[ test ]
fn mirror_substrate_returns_all_the_lost_energy()
{
  for e_avg in [ 0.2_f32, 0.5, 0.9, 1.0 ]
  {
    let f = multi_scatter_fresnel( [ 1.0, 1.0, 1.0 ], e_avg );
    for c in f
    {
      assert!( ( c - 1.0 ).abs() < 1e-4, "E_avg {e_avg}: expected 1, got {c}" );
    }
  }
}

/// A substrate that reflects nothing has nothing to give back.
#[ test ]
fn black_substrate_returns_nothing()
{
  let f = multi_scatter_fresnel( [ 0.0, 0.0, 0.0 ], 0.5 );
  for c in f
  {
    assert!( c.abs() < 1e-6, "expected 0, got {c}" );
  }
}

/// The series is the sum of repeatedly Fresnel-weighted bounces, so it never
/// returns more than the single-bounce weight `F_avg` it is replacing — the
/// reason the old single-`F_avg` weighting over-brightened rough metals.
#[ test ]
fn series_never_exceeds_the_single_bounce_weight()
{
  for e_avg in [ 0.1_f32, 0.4, 0.7, 0.95 ]
  {
    for f_avg in [ 0.04_f32, 0.2, 0.5, 0.8, 1.0 ]
    {
      let out = multi_scatter_fresnel( [ f_avg; 3 ], e_avg )[ 0 ];
      assert!
      (
        out <= f_avg + 1e-5,
        "E_avg {e_avg}, F_avg {f_avg}: series {out} exceeds the single-bounce weight"
      );
    }
  }
}

/// Each channel carries its own Fresnel, and the squared term saturates the
/// tint — a copper-ish substrate comes back *more* colored than `F_avg`, which
/// is exactly what the single-`F_avg` weighting used to wash out.
#[ test ]
fn colored_substrate_saturates()
{
  let f_avg = [ 0.95_f32, 0.64, 0.54 ];
  let out = multi_scatter_fresnel( f_avg, 0.6 );

  let ratio_in = f_avg[ 2 ] / f_avg[ 0 ];
  let ratio_out = out[ 2 ] / out[ 0 ];
  assert!( ratio_out < ratio_in, "tint did not saturate: {ratio_out} >= {ratio_in}" );
}

/// More single-scatter energy left on the table ( lower `E_avg` ) means a bigger
/// share of the response comes back through multiple bounces, so the weight
/// grows as `E_avg` falls — monotonically, with no inversion in the middle.
#[ test ]
fn weight_grows_as_the_surface_loses_more_energy()
{
  let mut previous = f32::INFINITY;
  for e_avg in [ 0.95_f32, 0.8, 0.6, 0.4, 0.2 ]
  {
    let out = multi_scatter_fresnel( [ 0.5; 3 ], e_avg )[ 0 ];
    assert!( out < previous, "E_avg {e_avg}: {out} did not fall below {previous}" );
    previous = out;
  }
}

/// An isotropic surface must index its own row: `alpha_t = alpha_b = r²` has to
/// come back as exactly `r`, which is what makes the effective roughness safe to
/// apply unconditionally rather than only under the anisotropy define.
#[ test ]
fn effective_roughness_is_identity_for_an_isotropic_lobe()
{
  for roughness in [ 0.05_f32, 0.2, 0.5, 0.8, 1.0 ]
  {
    let alpha = roughness * roughness;
    let effective = effective_roughness( alpha, alpha );
    assert!
    (
      ( effective - roughness ).abs() < 1e-5,
      "roughness {roughness} came back as {effective}"
    );
  }
}

/// An anisotropic lobe is matched by the isotropic one of equal projected slope
/// area — the geometric mean of the two alphas — so the effective roughness sits
/// between the roughnesses of the two axes, not at either end.
#[ test ]
fn effective_roughness_sits_between_the_two_axes()
{
  let alpha_t = 0.64_f32; // roughness 0.8 along the stretched axis
  let alpha_b = 0.04_f32; // roughness 0.2 along the narrow one
  let effective = effective_roughness( alpha_t, alpha_b );

  assert!( effective > 0.2 && effective < 0.8, "got {effective}, expected between the axes" );
  // sqrt( sqrt( 0.64 * 0.04 ) ) = sqrt( 0.16 ) = 0.4
  assert!( ( effective - 0.4 ).abs() < 1e-5, "got {effective}, expected the geometric mean 0.4" );
}

/// Monotone in both axes: stretching either one can only make the lobe rougher,
/// so the row the table is read from can only move in one direction.
#[ test ]
fn effective_roughness_grows_with_either_axis()
{
  let base = effective_roughness( 0.1, 0.1 );
  assert!( effective_roughness( 0.4, 0.1 ) > base );
  assert!( effective_roughness( 0.1, 0.4 ) > base );
}

/// Bounded to a real table row. A degenerate alpha — a mirror axis, or a
/// denormalised tangent frame — must not index outside the table or hand the
/// shader a `NaN` row.
#[ test ]
fn effective_roughness_stays_a_valid_table_row()
{
  for ( alpha_t, alpha_b ) in
  [
    ( 0.0_f32, 0.5_f32 ), ( 0.5, 0.0 ), ( -0.2, 0.5 ), ( 4.0, 4.0 ),
    ( f32::NAN, 0.5 ), ( f32::INFINITY, 0.5 ), ( 1.0, f32::NEG_INFINITY ),
  ]
  {
    let effective = effective_roughness( alpha_t, alpha_b );
    assert!( effective.is_finite(), "( {alpha_t}, {alpha_b} ) gave {effective}" );
    assert!( ( 0.0 ..= 1.0 ).contains( &effective ), "( {alpha_t}, {alpha_b} ) gave {effective}" );
  }
}
