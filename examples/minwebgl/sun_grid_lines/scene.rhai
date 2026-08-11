// Static visual configuration for the sun/grid HUD diagram — every color,
// opacity, and radius that isn't already keyboard-interactive (seed, node
// count, grid density stay live; see `Params` in src/main.rs). Evaluated by
// `src/scene.rs` at compile time via `include_str!`, run through
// `scene_script::build_engine()`, and extracted into `SceneConfig` via
// `rhai`'s serde bridge ( `rhai::serde::from_dynamic` ) — a rebuild is
// required to pick up edits, see readme.md.

let background = #{ top : [ 0.0196, 0.0549, 0.0941 ], bottom : [ 0.0549, 0.1490, 0.2392 ] };
let nebula = #{ color : [ 0.0706, 0.2000, 0.2902 ], opacity : 0.45 };
let stars = #{ color : [ 0.6275, 0.8980, 1.0 ], intensity : 0.6 };
let grid = #{ color : [ 0.3137, 0.5490, 0.7451 ], opacity : 0.18 };
let sun_corona = #{ inner : [ 1.0, 0.8941, 0.4392 ], mid : [ 1.0, 0.6824, 0.1020 ], outer : [ 1.0, 0.2314, 0.0 ] };
let sun_disc = #{ dark : [ 1.0, 0.4157, 0.0 ], mid : [ 1.0, 0.8941, 0.4392 ], bright : [ 1.0, 1.0, 1.0 ], base_radius : 0.075 };
let orbit_ring = #{ color : [ 0.3922, 0.8235, 1.0 ], radius : 0.425 };

#{
  background : background,
  nebula : nebula,
  stars : stars,
  grid : grid,
  sun_corona : sun_corona,
  sun_disc : sun_disc,
  orbit_ring : orbit_ring,
}
