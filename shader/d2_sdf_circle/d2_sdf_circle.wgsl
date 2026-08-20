//@ name: d2_sdf_circle
//@ description: Signed distance from a 2D point to a circle of the given radius.
//@ tags: category:sdf, dim:2d
//@ depends_on:
//@ export: fn d2_sdf_circle(p: vec2f, radius: f32) -> f32
//@ export: fn d2_sdf_circle_preview(p: vec2f, radius: f32) -> f32
//@ param: radius argument f32 range(0.05, 0.45)

fn d2_sdf_circle( p : vec2f, radius : f32 ) -> f32
{
  // Negative inside the disk, zero on the circle, positive outside.
  return length( p ) - radius;
}

fn d2_sdf_circle_preview( p : vec2f, radius : f32 ) -> f32
{
  return d2_sdf_circle( p, radius );
}
