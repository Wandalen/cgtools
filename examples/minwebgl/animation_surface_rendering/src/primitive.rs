mod private
{
  use minwebgl as gl;
  use gl::{ F32x2, geometry::BoundingBox };
  use std::cell::RefCell;
  use std::rc::Rc;
  use geometry_generation::AttributesData;
  use kurbo::PathEl;
  use crate::primitive_data::PrimitiveData;

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
    let mut positions = Vec::new();
    let mut indices = Vec::new();

    let half_width = width / 2.0;

    let mut add_segment =
    | start_point : &F32x2, end_point : &F32x2 |
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

    curve.windows( 2 )
    .for_each
    ( 
      | w |
      {
        let start_point = F32x2::from_array( w[ 0 ] );
        let end_point = F32x2::from_array( w[ 1 ] );
        add_segment( &end_point, &start_point );
      }
    );

    let start_point = ( *curve.last().unwrap() ).into();
    let end_point = F32x2::from_array( *curve.first().unwrap() );
    add_segment( &start_point, &end_point );

    let attributes = AttributesData
    {
      positions,
      indices
    };

    Some
    (
      PrimitiveData::new( Some( Rc::new( RefCell::new( attributes ) ) ) )
    )
  }

  /// Converts a vector of contours (closed paths) into a filled geometry.
  ///
  /// This function takes a set of contours, identifies the outer boundary
  /// (the largest contour), and then triangulates the resulting shape,
  /// accounting for any inner contours which act as holes.
  ///
  /// # Arguments
  ///
  /// * `contours` - A slice of vectors, where each inner vector represents a
  /// contour as a series of 2D points. The first contour is the outer body,
  /// subsequent ones are holes.
  ///
  /// # Returns
  ///
  /// An `Option<PrimitiveData>` containing the triangulated geometry for the
  /// filled shape. Returns `None` if the input is empty or invalid.
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
      let x1 = contour.iter()
      .map( | [ x, _ ] | x )
      .min_by( | x, y | x.total_cmp( y ) ).unwrap();
      let y1 = contour.iter()
      .map( | [ _, y ] | y )
      .min_by( | x, y | x.total_cmp( y ) ).unwrap();
      let x2 = contour.iter()
      .map( | [ x, _ ] | x )
      .max_by( | x, y | x.total_cmp( y ) ).unwrap();
      let y2 = contour.iter()
      .map( | [ _, y ] | y )
      .max_by( | x, y | x.total_cmp( y ) ).unwrap();
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
      .clone()
      .into_iter()
      .flatten()
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

    let primitive_data = PrimitiveData::new( Some( Rc::new( RefCell::new( attributes ) ) ) );

    Some( primitive_data )
  }

  /// Converts a vector of 2D points into a `Vec<PathEl>`.
  ///
  /// The first point is converted to a `PathEl::MoveTo` and subsequent points
  /// are converted to `PathEl::LineTo`.
  ///
  /// # Arguments
  ///
  /// * `points` - A `Vec` of 2D points `[ f32; 2 ]`.
  ///
  /// # Returns
  ///
  /// A `Vec<PathEl>` representing the path.
  pub fn points_to_path( points : Vec< [ f32; 2 ] > ) -> Vec< PathEl >
  {
    let mut points = points.into_iter()
    .map
    (
      | [ x, y ] |
      {
        PathEl::LineTo( kurbo::Point::new( x as f64, y as f64 ) )
      }
    )
    .collect::< Vec< _ > >();

    if let Some( el ) = points.get_mut( 0 )
    {
      if let PathEl::LineTo( p ) = el.clone()
      {
        *el = PathEl::MoveTo( p );
      }
    }

    points
  }

  /// Converts a `Vec<PathEl>` into a flattened vector of 2D points.
  ///
  /// This function uses `kurbo::flatten` to convert a path with curves
  /// into a series of straight line segments. The tolerance for flattening is
  /// set to `0.25`.
  ///
  /// # Arguments
  ///
  /// * `path` - A `Vec<PathEl>` representing the path to flatten.
  ///
  /// # Returns
  ///
  /// A `Vec<[f32; 2]>` containing the flattened 2D points of the path.
  pub fn path_to_points( path : Vec< PathEl > ) -> Vec< [ f32; 2 ] >
  {
    let mut points = vec![];

    kurbo::flatten
    (
      kurbo::BezPath::from_vec( path ),
      0.25,
      | el |
      {
        let point = match el
        {
          PathEl::MoveTo( p) | PathEl::LineTo( p ) =>
          {
            [ p.x as f32, p.y as f32 ]
          },
          _ => unreachable!( "kurbo::flatten can only return MoveTo and LineTo PathEls" )
        };
        points.push( point );
      }
    );

    points
  }
}

::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;

  orphan use
  {
    curve_to_geometry,
    contours_to_fill_geometry,
    points_to_path,
    path_to_points
  };
}
