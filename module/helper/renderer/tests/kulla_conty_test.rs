//! Verifies the Kulla–Conty multi-scatter energy-compensation table generator
//! ( `renderer::webgl::loaders::kulla_conty::kulla_conty_tables` ) — the pure,
//! off-GPU computation of the GGX directional albedo `E(μ, α)` and hemisphere
//! average `E_avg(α)`, later uploaded as a LUT for direct-light energy
//! compensation. Pure math, natively testable.

use renderer::webgl::loaders::kulla_conty::{ KullaContyTables, kulla_conty_tables };

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
