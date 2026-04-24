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
  }
}

#[ cfg( test ) ]
mod tests
{
  use super::private::*;

  #[ test ]
  fn default_centers_origin()
  {
    let cam = Camera::default();
    let ( x, y ) = cam.project( ( 0.0, 0.0 ) );
    assert!( ( x - 400.0 ).abs() < 1e-3, "expected x ~= 400, got {x}" );
    assert!( ( y - 300.0 ).abs() < 1e-3, "expected y ~= 300, got {y}" );
  }

  #[ test ]
  fn translate_shifts_projection()
  {
    let cam = Camera { world_center : ( 100.0, 0.0 ), ..Camera::default() };
    let ( x, y ) = cam.project( ( 0.0, 0.0 ) );
    assert!( ( x - 300.0 ).abs() < 1e-3, "translate didn't shift x correctly: {x}" );
    assert!( ( y - 300.0 ).abs() < 1e-3, "translate changed y unexpectedly: {y}" );
  }

  #[ test ]
  fn zoom_scales_distance_from_center()
  {
    let cam = Camera { zoom : 2.0, ..Camera::default() };
    let ( x_zoomed, _ ) = cam.project( ( 50.0, 0.0 ) );
    let cam_one = Camera::default();
    let ( x_one, _ ) = cam_one.project( ( 50.0, 0.0 ) );
    // Distance from viewport centre should be doubled under 2x zoom.
    let d_zoomed = x_zoomed - 400.0;
    let d_one    = x_one - 400.0;
    assert!( ( d_zoomed / d_one - 2.0 ).abs() < 1e-3, "zoom distance ratio: {d_zoomed} / {d_one}" );
  }
}

mod_interface::mod_interface!
{
  exposed use Camera;
}
