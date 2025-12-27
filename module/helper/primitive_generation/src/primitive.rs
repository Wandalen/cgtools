//! This module store functions and structures for creating `PrimitiveData`
//! of different abstactions like curves.

#[ allow( clippy::too_many_lines ) ]
mod private
{
  use minwebgl as gl;
  use gl::{ F32x2, F32x4, geometry::BoundingBox };
  use std::cell::RefCell;
  use std::rc::Rc;
  use crate::
  {
    AttributesData,
    PrimitiveData,
    Transform
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

  /// Converts a set of 2D contours into a solid flat 3D primitive suitable for rendering.
  ///
  /// The function first identifies the largest contour, which it assumes is the
  /// main body of the shape. It then classifies other contours as either holes
  /// (if they are fully contained within the main body's bounding box) or as
  /// separate bodies. Finally, it uses a triangulation algorithm (`earcutr`) to
  /// generate a filled mesh with positions and indices.
  ///
  /// # Arguments
  ///
  /// * `contours`: A slice of vectors, where each inner vector is a contour
  ///   represented by an array of `[f32; 2]` points.
  ///
  /// # Returns
  ///
  /// Returns `Some( PrimitiveData )` on success, containing the generated mesh
  /// data. Returns `None` if the input `contours` is empty or if the
  /// triangulation process fails.
  pub fn contours_to_fill_geometry( contours : &[ Vec< [ f32; 2 ] > ] ) -> Option< PrimitiveData >
  {
    if contours.is_empty()
    {
      return None;
    }

    let mut body_id = 0;
    let mut max_box_diagonal_size = 0.0;
    for ( i, contour ) in contours.iter().enumerate()
    {
      if contour.is_empty()
      {
        continue;
      }
      let ( mut x1, mut y1, mut x2, mut y2 ) = ( f32::MAX, f32::MAX, f32::MIN, f32::MIN );
      for [ x, y ] in contour
      {
        x1 = x1.min( *x );
        y1 = y1.min( *y );
        x2 = x2.max( *x );
        y2 = y2.max( *y );
      }
      let controur_size = ( ( x2 - x1 ).powi( 2 ) + ( y2 - y1 ).powi( 2 ) ).sqrt();
      if max_box_diagonal_size < controur_size
      {
        max_box_diagonal_size = controur_size;
        body_id = i;
      }
    }

    let body_bounding_box = BoundingBox::compute2d
    (
      contours.get( body_id ).unwrap()
      .iter()
      .flatten()
      .cloned()
      .collect::< Vec< _ > >()
      .as_slice()
    );

    let mut outside_body_list = vec![];
    let mut inside_body_list = vec![];
    for ( i, contour ) in contours.iter().enumerate()
    {
      if body_id == i
      {
        continue;
      }

      let bounding_box = BoundingBox::compute2d
      (
        contour
        .iter()
        .flatten()
        .cloned()
        .collect::< Vec< _ > >()
        .as_slice()
      );

      let has_part_outside_body = bounding_box.left() < body_bounding_box.left() ||
      bounding_box.right() > body_bounding_box.right() ||
      bounding_box.up() > body_bounding_box.up() ||
      bounding_box.down() < body_bounding_box.down();

      if has_part_outside_body
      {
        outside_body_list.push( contour.clone() );
      }
      else
      {
        inside_body_list.push( contour.clone() );
      }
    }

    let mut base = vec![ contours[ body_id ].clone() ];
    base.extend( inside_body_list );

    let mut bodies = vec![ base ];
    bodies.extend( outside_body_list.into_iter().map( | c | vec![ c ] ) );

    let mut positions = vec![];
    let mut indices = vec![];

    for contours in bodies
    {
      let mut flat_positions: Vec< f64 > = Vec::new();
      let mut hole_indices: Vec< usize > = Vec::new();

      if let Some( outer_contour ) = contours.get( 0 )
      {
        if outer_contour.is_empty()
        {
          return None;
        }
        for &[ x, y ] in outer_contour
        {
          flat_positions.push( x as f64 );
          flat_positions.push( y as f64 );
        }
      }
      else
      {
        return None;
      }

      // Process holes (remaining contours)
      // Their winding order must be opposite to the outer (e.g., CW for holes)
      for i in 1..contours.len()
      {
        let hole_contour = &contours[ i ];
        if hole_contour.is_empty()
        {
          continue;
        }

        hole_indices.push( flat_positions.len() / 2 );

        for &[ x, y ] in hole_contour
        {
          flat_positions.push( x as f64 );
          flat_positions.push( y as f64 );
        }
      }

      // Perform triangulation
      let Ok( body_indices ) = earcutr::earcut( &flat_positions, &hole_indices, 2 )
      else
      {
        continue;
      };

      let body_indices = body_indices.into_iter()
      .map( | i | i as u32 )
      .collect::< Vec< _ > >();

      let body_positions = flat_positions.chunks( 2 )
      .map( | c | [ c[ 0 ] as f32, c[ 1 ] as f32, 0.0 ] )
      .collect::< Vec< _ > >();

      let positions_count = positions.len();
      positions.extend( body_positions );
      indices.extend
      (
        body_indices.iter()
        .map( | i | i + positions_count as u32 )
      );
    }

    let attributes = AttributesData
    {
      positions,
      indices,
    };

    let primitive_data = PrimitiveData
    {
      attributes : Rc::new( RefCell::new( attributes ) ),
      color : F32x4::default(),
      transform : Transform::default()
    };

    Some( primitive_data )
  }

  /// Creates a unit plane (1×1) centered at the origin.
  ///
  /// Use `primitive.transform.scale = F32x3::new(width, height, 1.0);`
  /// to adjust its size later.
  ///
  /// The plane lies in the XY plane (Z = 0).
  pub fn plane_to_geometry() -> Option< PrimitiveData >
  {
    let positions =
    vec![
      [ -0.5, -0.5, 0.0 ],
      [ 0.5, -0.5, 0.0 ],
      [ 0.5,  0.5, 0.0 ],
      [ -0.5,  0.5, 0.0 ]
    ];

    let indices =
    vec![
      0, 1, 2,
      0, 2, 3
    ];

    let attributes = AttributesData
    {
      positions,
      indices
    };

    Some
    (
      PrimitiveData
      {
        attributes : Rc::new( RefCell::new( attributes ) ),
        color : F32x4::default(),
        transform : Transform::default(),
      }
    )
  }
}

crate::mod_interface!
{
  orphan use
  {
    curve_to_geometry,
    contours_to_fill_geometry,
    plane_to_geometry
  };
}
