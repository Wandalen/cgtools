mod private
{
  use minwebgl as gl;
  use gl::{ F32x2, F32x4 };
  use std::cell::RefCell;
  use std::rc::Rc;
  use crate::
  { 
    AttributesData, PrimitiveData, Transform 
  };

  /// Converts a `&[ [ f32; 2 ] ]` into `GeometryData` representing its 2D outline
  /// as a series of rectangles, each with a specified `width`.
  ///
  /// This function flattens the Bezier path into many small line segments.
  /// For each segment, it constructs a rectangle of the given `width` centered
  /// on the segment. These rectangles are then triangulated (into two triangles)
  /// and added to the `GeometryData`.
  ///
  /// # Arguments
  ///
  /// * `curve` - The `&[ [ f32; 2 ] ]` to convert into a thick geometry.
  /// * `width` - The desired thickness of the stroked path.
  ///
  /// # Returns
  ///
  /// A `GeometryData` struct containing the 3D vertex positions and triangle indices
  /// that form the rectangular segments of the path. The Z-coordinate is always 0.0.
  pub fn curve_to_geometry( curve : &[ [ f32; 2 ] ], width : f32 ) -> Option< PrimitiveData > 
  {
    let Some( mut start_point )  = curve.first()
    .map( | p | F32x2::from_array( *p ) )
    else
    {
      return None;
    };

    let mut positions = Vec::new();
    let mut indices = Vec::new();

    let half_width = width / 2.0;

    let mut add_segment = 
    | end_point : &F32x2, start_point : &F32x2 |
    {
      let direction = ( *end_point - *start_point ).normalize();

      let normal = F32x2::new( -direction.y(), direction.x() );

      let p0 = *start_point - normal * half_width;
      let p1 = *start_point + normal * half_width;
      let p2 = *end_point + normal * half_width;
      let p3 = *end_point - normal * half_width;

      let base_idx = positions.len() as u32;

      positions.push( [ p0.x(), p0.y(), 0.0 ] );
      positions.push( [ p1.x(), p1.y(), 0.0 ] );
      positions.push( [ p2.x(), p2.y(), 0.0 ] );
      positions.push( [ p3.x(), p3.y(), 0.0 ] );

      indices.push( base_idx + 0 );
      indices.push( base_idx + 1 );
      indices.push( base_idx + 2 );

      indices.push( base_idx + 0 );
      indices.push( base_idx + 2 );
      indices.push( base_idx + 3 );
    };

    let mut i = 1;
    while i < curve.len()
    {
      let end_point = F32x2::from_array( curve[ i ] );
      add_segment( &end_point, &start_point );
      start_point = end_point;
      i += 1;
    }

    start_point = ( *curve.last().unwrap() ).into();
    let end_point = F32x2::from_array( *curve.first().unwrap() );
    add_segment( &end_point, &start_point );

    let attributes = AttributesData
    {
      positions, 
      indices
    };

    Some(
      PrimitiveData 
      { 
        attributes : Rc::new( RefCell::new( attributes ) ),
        color : F32x4::default(),
        transform: Transform::default(),
      }
    )
  }
}

crate::mod_interface!
{
  orphan use
  {
    curve_to_geometry
  };
}
