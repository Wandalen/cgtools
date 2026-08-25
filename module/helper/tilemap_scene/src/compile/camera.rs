//! `Camera` — how the compile layer projects world pixels onto the viewport.
//!
//! Slice 1 supports translate + uniform zoom only. Rotation, per-axis zoom,
//! and parallax land with viewport-anchored objects in a later slice.

mod private
{
  /// A 2D camera with centre, uniform zoom, and viewport size.
  ///
  /// Applied to a world-pixel `( wx, wy )`:
  ///
  /// ```text
  /// screen_x = (wx - world_center.0) * zoom + viewport_size.0 / 2
  /// screen_y = (wy - world_center.1) * zoom + viewport_size.1 / 2
  /// ```
  ///
  /// Both the input and output are Y-up (`hex_to_world_pixel_*` already
  /// applied the flip from `tiles_tools`' Y-down convention).
  #[ derive( Debug, Clone, Copy ) ]
  pub struct Camera
  {
    /// World-space centre of the camera view, in pixels.
    pub world_center : ( f32, f32 ),
    /// Uniform zoom factor. `1.0` = 1 world pixel per screen pixel.
    pub zoom : f32,
    /// Output viewport size in screen pixels, `( width, height )`.
    pub viewport_size : ( u32, u32 ),
  }

  impl Default for Camera
  {
    #[ inline ]
    fn default() -> Self
    {
      Self
      {
        world_center : ( 0.0, 0.0 ),
        zoom : 1.0,
        viewport_size : ( 800, 600 ),
      }
    }
  }

  impl Camera
  {
    /// Project a world-space pixel position onto the viewport.
    #[ inline ]
    #[ must_use ]
    pub fn project( &self, world : ( f32, f32 ) ) -> ( f32, f32 )
    {
      let ( wx, wy ) = world;
      let ( cx, cy ) = self.world_center;
      let vx = self.viewport_size.0 as f32 * 0.5;
      let vy = self.viewport_size.1 as f32 * 0.5;
      ( ( wx - cx ) * self.zoom + vx, ( wy - cy ) * self.zoom + vy )
    }

    /// The same world→screen projection as [`Self::project`], expressed as a
    /// **column-major** 3×3 matrix (matching `tilemap_renderer`'s
    /// `Transform::to_mat3` layout, element `(row i, col j)` at index `j*3+i`).
    ///
    /// A GPU backend uploads this once per frame and applies it to world-space
    /// draws, so per-instance transforms stay in world space and a pan/zoom is a
    /// single uniform change rather than a re-projection of every sprite.
    /// Because the camera is translate + uniform zoom only, the linear part is a
    /// scalar `zoom` and the translation carries the centre offset:
    /// `screen = zoom * world + ( viewport/2 - zoom * center )`.
    #[ inline ]
    #[ must_use ]
    pub fn to_view_mat3( &self ) -> [ f32; 9 ]
    {
      let ( cx, cy ) = self.world_center;
      let z = self.zoom;
      let tx = self.viewport_size.0 as f32 * 0.5 - z * cx;
      let ty = self.viewport_size.1 as f32 * 0.5 - z * cy;
      [
        z,   0.0, 0.0,
        0.0, z,   0.0,
        tx,  ty,  1.0,
      ]
    }
  }
}

mod_interface::mod_interface!
{
  exposed use Camera;
}
