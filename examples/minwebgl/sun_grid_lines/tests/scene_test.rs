//! Native round-trip test for the `scene` module — `scene.rhai` parsing via
//! `SceneConfig::load()`.
//!
//! Relocated from `src/scene.rs`, per the all-tests-in-tests/ convention.

use minwebgl_sun_grid_lines::scene::SceneConfig;

/// Asserts every field `scene.rhai` declares round-trips through
/// `SceneConfig::load()` correctly — deliberately exhaustive ( not just a
/// spot check ) so every field is exercised on every target, including
/// native, where `main.rs`'s wasm32-gated `app_run()` — the only other
/// consumer — never compiles in and can't do it instead.
#[ test ]
#[ expect( clippy::float_cmp, reason = "every value here is a literal parsed straight out of scene.rhai with no arithmetic in between, so bit-exact round-trip fidelity is exactly what this test means to check" ) ]
fn scene_rhai_parses_and_matches_known_values()
{
  let scene = SceneConfig::load();

  assert_eq!( scene.background.top.to_array(), [ 0.0196, 0.0549, 0.0941 ] );
  assert_eq!( scene.background.bottom.to_array(), [ 0.0549, 0.1490, 0.2392 ] );

  assert_eq!( scene.nebula.color.to_array(), [ 0.0706, 0.2000, 0.2902 ] );
  assert_eq!( scene.nebula.opacity, 0.45 );

  assert_eq!( scene.stars.color.to_array(), [ 0.6275, 0.8980, 1.0 ] );
  assert_eq!( scene.stars.intensity, 0.6 );

  assert_eq!( scene.grid.color.to_array(), [ 0.3137, 0.5490, 0.7451 ] );
  assert_eq!( scene.grid.opacity, 0.18 );

  assert_eq!( scene.sun_corona.inner.to_array(), [ 1.0, 0.8941, 0.4392 ] );
  assert_eq!( scene.sun_corona.mid.to_array(), [ 1.0, 0.6824, 0.1020 ] );
  assert_eq!( scene.sun_corona.outer.to_array(), [ 1.0, 0.2314, 0.0 ] );

  assert_eq!( scene.sun_disc.dark.to_array(), [ 1.0, 0.4157, 0.0 ] );
  assert_eq!( scene.sun_disc.mid.to_array(), [ 1.0, 0.8941, 0.4392 ] );
  assert_eq!( scene.sun_disc.bright.to_array(), [ 1.0, 1.0, 1.0 ] );
  assert_eq!( scene.sun_disc.base_radius, 0.075 );

  assert_eq!( scene.orbit_ring.color.to_array(), [ 0.3922, 0.8235, 1.0 ] );
  assert_eq!( scene.orbit_ring.radius, 0.425 );
}
