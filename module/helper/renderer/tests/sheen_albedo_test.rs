//! Verifies the fuzz ( sheen ) directional-albedo table generator
//! ( `renderer::webgl::loaders::sheen_albedo` ) — the pure, off-GPU integration
//! of the Charlie NDF / Ashikhmin-Premoze visibility pair that `main.frag`
//! evaluates, uploaded as a LUT so the substrate under the fuzz layer can be
//! albedo-scaled.
//!
//! Spec reference ( <https://academysoftwarefoundation.github.io/OpenPBR/> ):
//! OpenPBR layers fuzz over the base as `layer( base, fuzz, fuzz_weight )`, i.e.
//! the layer takes its energy *from* the substrate; the glTF spelling of the
//! same statement is `f = f_fuzz + f_base * ( 1 - max3( fuzz_color ) * E( mu ) )`.
//! `E` is that directional albedo, which has no closed form for the Charlie NDF.
//!
//! Zero `WebGl2RenderingContext` calls — the same off-GPU pattern as
//! `kulla_conty_test.rs`.

#![ cfg( not( target_arch = "wasm32" ) ) ]

use renderer::webgl::loaders::sheen_albedo::
{
  FUZZ_ALPHA_MIN,
  SheenAlbedoTable,
  sheen_albedo_table,
  sheen_directional_albedo,
};

/// Cheap-but-accurate table for tests.
fn table() -> SheenAlbedoTable
{
  sheen_albedo_table( 16, 16, 64, 64 )
}

#[ test ]
fn table_shape_is_consistent()
{
  let t = table();
  assert_eq!( t.uv_samples, 16 );
  assert_eq!( t.roughness_samples, 16 );
  assert_eq!( t.e.len(), 16 * 16 );
}

/// `E` is an albedo: every entry stays inside `[ 0, 1 ]`, so the substrate
/// scaling `1 - max3( fuzz_color ) * E` can never turn the base negative.
#[ test ]
fn albedo_is_bounded()
{
  for ( i, e ) in table().e.iter().enumerate()
  {
    assert!( e.is_finite(), "entry {i} is not finite: {e}" );
    assert!( ( 0.0 ..= 1.0 ).contains( e ), "entry {i} outside [0, 1]: {e}" );
  }
}

/// Fuzz is a grazing-angle effect: the lobe reflects far more of what reaches it
/// at the silhouette than head-on, for every roughness. This is the whole reason
/// the scaling has to be per-pixel rather than a constant factor.
#[ test ]
fn albedo_falls_off_from_grazing_to_head_on()
{
  for alpha in [ 0.1_f32, 0.3, 0.6, 1.0 ]
  {
    let grazing = sheen_directional_albedo( 0.0, alpha, 64, 64 );
    let mid = sheen_directional_albedo( 0.5, alpha, 64, 64 );
    let head_on = sheen_directional_albedo( 1.0, alpha, 64, 64 );
    assert!( grazing > mid, "alpha {alpha}: grazing {grazing} <= mid {mid}" );
    assert!( mid > head_on, "alpha {alpha}: mid {mid} <= head-on {head_on}" );
  }
}

/// A head-on view of the fuzz layer sees almost none of it, so the substrate is
/// left essentially untouched there — a fuzzy material must not read as a
/// uniformly dimmed one.
#[ test ]
fn head_on_albedo_is_negligible()
{
  for alpha in [ 0.1_f32, 0.5, 1.0 ]
  {
    let head_on = sheen_directional_albedo( 1.0, alpha, 64, 64 );
    assert!( head_on < 0.2, "alpha {alpha}: head-on albedo {head_on} is not negligible" );
  }
}

/// The shader floors the fuzz roughness at [`FUZZ_ALPHA_MIN`] before it reaches
/// `D_Charlie`, so the table has to be built with the same floor: below it every
/// row is the `FUZZ_ALPHA_MIN` row, and the albedo it reports is the albedo of
/// the lobe actually evaluated.
#[ test ]
fn roughness_below_the_floor_reuses_the_floor_row()
{
  let floored = sheen_directional_albedo( 0.3, FUZZ_ALPHA_MIN, 64, 64 );
  for alpha in [ 0.0_f32, 0.01, 0.05, FUZZ_ALPHA_MIN ]
  {
    let e = sheen_directional_albedo( 0.3, alpha, 64, 64 );
    assert!
    (
      ( e - floored ).abs() < 1e-6,
      "alpha {alpha} gave {e}, expected the floored {floored}"
    );
  }
}

/// A rougher fuzz layer spreads the same energy wider, so it loses the sharp
/// grazing rim and gains at mid angles. Both halves of that trade are asserted
/// because only the pair pins the shape of the curve.
#[ test ]
fn roughness_widens_the_lobe()
{
  let grazing_narrow = sheen_directional_albedo( 0.0, 0.1, 64, 64 );
  let grazing_wide = sheen_directional_albedo( 0.0, 1.0, 64, 64 );
  assert!( grazing_narrow > grazing_wide, "{grazing_narrow} <= {grazing_wide}" );

  let mid_narrow = sheen_directional_albedo( 0.7, 0.1, 64, 64 );
  let mid_wide = sheen_directional_albedo( 0.7, 1.0, 64, 64 );
  assert!( mid_wide > mid_narrow, "{mid_wide} <= {mid_narrow}" );
}

/// The integration is a deterministic quadrature, not a stochastic estimate, so
/// the same inputs give bit-identical output — the property the regression
/// checks above lean on, and what lets the LUT be compared across runs.
#[ test ]
fn integration_is_deterministic()
{
  let a = sheen_albedo_table( 8, 8, 32, 32 );
  let b = sheen_albedo_table( 8, 8, 32, 32 );
  assert_eq!( a, b );
}

/// The table is the grid of `sheen_directional_albedo`, laid out row-major with
/// the roughness on the row axis — the layout `main.frag` samples with
/// `vec2( mu, alpha )`, shared with the Kulla–Conty LUT.
#[ test ]
fn table_layout_matches_the_sampling_convention()
{
  let t = sheen_albedo_table( 5, 4, 32, 32 );
  for r in 0 .. t.roughness_samples
  {
    let alpha = r as f32 / ( t.roughness_samples - 1 ) as f32;
    for u in 0 .. t.uv_samples
    {
      let mu = u as f32 / ( t.uv_samples - 1 ) as f32;
      let expected = sheen_directional_albedo( mu, alpha, 32, 32 );
      let actual = t.e[ r * t.uv_samples + u ];
      assert!
      (
        ( actual - expected ).abs() < 1e-6,
        "cell ( r {r}, u {u} ): {actual} != {expected}"
      );
    }
  }
}
