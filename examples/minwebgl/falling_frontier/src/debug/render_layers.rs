//! Render-layer switches - which parts of the scene the frame loop draws.
//! Every field but `show_trajectories` is a row of the Render Layers dev
//! panel (`layers_panel`); held as `GridTuning::layers`, so the panel, the
//! frame loop and the pick pass all read the one shared tuning state.

/// The scene layers (grid, background, starfield, asteroids, ships,
/// station) are one switch per draw pass in `main.rs`'s frame closure, so
/// any combination of them can be shown alone or hidden alone (e.g. "only
/// the grid", "everything but asteroids"). The other switches change how
/// those passes look rather than drawing anything by themselves - see
/// `layers_panel`'s module doc for how the panel groups them.
///
/// `lighting_enabled` is deliberately separate from `shadows_enabled`: the
/// former drops `hull.frag` to a flat unlit `u_color` (see hull.frag's
/// `u_lighting_enabled` branch), the latter only gates the shadow-map
/// sample within the normal lit path. `show_asteroids`/`show_ships`/
/// `show_station` also gate that object's contribution to the shadow-caster
/// pass and the pick pass, not just its own visible draw - a hidden object
/// shouldn't still cast a shadow or take a click.
#[ derive( Clone, Copy, Debug, PartialEq, Eq ) ]
pub struct RenderLayers
{
  /// Defaults to `true`, as the three.js original's grid toggle started on.
  pub show_grid : bool,
  /// The grid pass's selection focus around a selected ship: the view-zone
  /// ribbon together with the in-zone brightening and the asteroid glow,
  /// which `grid.frag` computes in the same branch. Part of the grid pass,
  /// so it has no effect while `show_grid` is off.
  pub show_view_ribbon : bool,
  pub show_background : bool,
  pub show_starfield : bool,
  pub show_asteroids : bool,
  pub show_ships : bool,
  pub show_station : bool,
  pub show_gizmo : bool,
  pub lighting_enabled : bool,
  pub shadows_enabled : bool,
  /// CRT scanline overlay - pure DOM/CSS effect (see `hud.rs`'s `ff-scanlines`
  /// element), not a WebGL draw call, but tracked here anyway so it lives in
  /// the same single Render Layers menu as every other switch instead of
  /// needing its own separate on/off surface.
  pub show_scanlines : bool,
  /// Trajectory ribbons. Defaults to `false`, as the three.js original also
  /// hid its trajectory group by default, and has no panel row: the
  /// feature is still unfinished.
  pub show_trajectories : bool,
}

impl Default for RenderLayers
{
  fn default() -> Self
  {
    Self
    {
      show_grid : true,
      show_view_ribbon : true,
      show_background : true,
      show_starfield : true,
      show_asteroids : true,
      show_ships : true,
      show_station : true,
      show_gizmo : true,
      lighting_enabled : true,
      shadows_enabled : true,
      show_scanlines : false,
      show_trajectories : false,
    }
  }
}
