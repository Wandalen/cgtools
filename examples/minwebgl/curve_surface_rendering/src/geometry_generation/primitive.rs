use minwebgl as gl;
use gl::F32x4;
use std::cell::RefCell;
use std::rc::Rc;
use crate::primitive::*;
use kurbo::
{
  Vec2,
  Point,
  BezPath
};

/// Converts a `kurbo::BezPath` into `GeometryData` representing its 2D outline
/// as a series of rectangles, each with a specified `width`.
///
/// This function flattens the Bezier path into many small line segments.
/// For each segment, it constructs a rectangle of the given `width` centered
/// on the segment. These rectangles are then triangulated (into two triangles)
/// and added to the `GeometryData`.
///
/// # Arguments
///
/// * `path` - The `kurbo::BezPath` to convert into a thick geometry.
/// * `width` - The desired thickness of the stroked path.
///
/// # Returns
///
/// A `GeometryData` struct containing the 3D vertex positions and triangle indices
/// that form the rectangular segments of the path. The Z-coordinate is always 0.0.
pub fn curve_to_geometry( path : BezPath, width : f32, color : F32x4 ) -> PrimitiveData 
{
  let mut positions = Vec::new();
  let mut indices = Vec::new();

  const FLATTEN_TOLERANCE: f64 = 0.5;

  let half_width = width as f64 / 2.0;
  let mut current_segment_start : Option< Point > = None;

  let callback = 
  | segment_el |
  {
    match segment_el 
    {
      kurbo::PathEl::MoveTo( p ) => 
      {
        current_segment_start = Some( p );
      }
      kurbo::PathEl::LineTo( end_point ) => 
      {
        if let Some(start_point) = current_segment_start {
          let direction = ( end_point - start_point ).normalize();

          let normal = Vec2::new(-direction.y, direction.x);

          let p0 = start_point - normal * half_width;
          let p1 = start_point + normal * half_width;
          let p2 = end_point + normal * half_width;
          let p3 = end_point - normal * half_width;

          let base_idx = positions.len() as u32;

          positions.push([p0.x as f32, p0.y as f32, 0.0]);
          positions.push([p1.x as f32, p1.y as f32, 0.0]);
          positions.push([p2.x as f32, p2.y as f32, 0.0]);
          positions.push([p3.x as f32, p3.y as f32, 0.0]);

          indices.push(base_idx + 0);
          indices.push(base_idx + 1);
          indices.push(base_idx + 2);

          indices.push(base_idx + 0);
          indices.push(base_idx + 2);
          indices.push(base_idx + 3);

          current_segment_start = Some(end_point);
        }
      }
      _ => unreachable!( "flatten() should only produce MoveTo and LineTo elements" ),
    }
  };

  kurbo::flatten( path, FLATTEN_TOLERANCE, callback ); 

  let attributes = AttributesData
  {
    positions, 
    indices
  };

  PrimitiveData 
  { 
    attributes : Rc::new( RefCell::new( attributes ) ),
    color,
    transform: Transform::default(),
  }
}