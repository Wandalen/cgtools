use std::{ cell::RefCell, collections::HashMap, i32, rc::Rc };
use tiles_tools::
{
  coordinates::*,
  mesh,
  layout::{ HexLayout, Orientation },
  patterns::{ Parity, ShiftedRectangleIter },
};
use minwebgl as gl;
use gl::
{
  math::{ d2::mat2x2h, F32x2, IntoVector },
  Program,
  JsCast,
  canvas::HtmlCanvasElement,
  geometry,
};
use web_sys::{ wasm_bindgen::prelude::Closure, MouseEvent };

fn main() -> Result< (), gl::WebglError >
{
  run()
}

fn run() -> Result< (), minwebgl::WebglError >
{
  gl::browser::setup( Default::default() );
  let o = gl::context::ContexOptions::new().reduce_dpr( true );
  let context = gl::context::retrieve_or_make_with( o )?;
  let canvas = context.canvas().unwrap().dyn_into::< HtmlCanvasElement >().unwrap();
  // used to scale canvas true size to css size
  let dpr = web_sys::window().unwrap().device_pixel_ratio() as f32;
  let canvas_size = ( canvas.width() as f32, canvas.height() as f32 ).into_vector() / dpr;

  // just abstract size in world space, it may be any units
  // size of a hexagon ( from center to vertex )
  let size = 0.1;
  // how to shift the hexagons to form a rectangle
  let shift_type = Parity::Odd;
  // orientation of hex can be either pointing upward or flat
  let orientation = Orientation::Pointy;
  // orientation of the hexagons
  let layout = HexLayout { orientation, size };
  // grid size
  let grid_size = [ 11, 13 ];
  let mut map = tiles_tools::grid::HexGrid::new( Default::default(), layout );
  for coord in ShiftedRectangleIter::new( grid_size, shift_type, layout )
  {
    map.map_mut().insert( coord, false );
  }

  let aspect = canvas_size[ 1 ] / canvas_size[ 0 ];
  let scale = 1.0;
  let aspect_scale : F32x2 = [ aspect * scale, scale ].into();
  let scale_m = mat2x2h::scale( aspect_scale.0 );
  // determine the center of the grid
  // to shift it to the center of the canvas
  let grid_center : F32x2 = layout.grid_center( ShiftedRectangleIter::new( grid_size, shift_type, layout ) ).into();
  let translation = mat2x2h::translate( [ -grid_center.x(), grid_center.y() ] );
  let mvp = scale_m * translation;

  let outline_mesh = mesh::grid_line_mesh( map.map().keys().map( | v | *v ), &layout, None );
  let outline_geometry = geometry::Positions::new
  (
    context.clone(),
    &outline_mesh,
    2
  )?;
  let filling_mesh = mesh::grid_triangle_mesh( map.map().keys().map( | v | *v ), &layout, None );
  let filling_geometry = geometry::Positions::new
  (
    context.clone(),
    &filling_mesh,
    2
  )?;
  let hex_mesh = mesh::hex_triangle_mesh( &layout );
  let hex_geometry = geometry::Positions::new
  (
    context.clone(),
    &hex_mesh,
    2
  )?;

  let vert = include_str!( "../../hexagonal_grid/src/shaders/main.vert" );
  let frag = include_str!( "../../hexagonal_grid/src/shaders/main.frag" );
  let hex_shader = Program::new( context.clone(), vert, frag )?;
  hex_shader.activate();

  context.clear_color( 0.9, 0.9, 0.9, 1.0 );
  context.clear( gl::COLOR_BUFFER_BIT );
  hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
  hex_shader.uniform_upload( "u_color", &[ 0.7, 0.7, 0.7, 1.0 ] );
  filling_geometry.activate();
  context.draw_arrays( gl::TRIANGLES, 0, filling_geometry.nvertices );
  hex_shader.uniform_upload( "u_color", &[ 0.1, 0.1, 0.1, 1.0 ] );
  outline_geometry.activate();
  context.draw_arrays( gl::LINES, 0, outline_geometry.nvertices );

  let map = Rc::new( RefCell::new( map ) );

  let mouse_move =
  {
    let map = map.clone();
    let context = context.clone();
    let canvas = canvas.clone();
    move | e : MouseEvent |
    {
      let rect = canvas.get_bounding_client_rect();
      let canvas_pos = F32x2::new( rect.left() as f32, rect.top() as f32 );
      let half_size : F32x2 = canvas_size / 2.0;
      let cursor_pos = F32x2::new( e.client_x() as f32, e.client_y() as f32 );
      // normalize coodinates to [ -1 : 1 ], then apply inverse ascpect scale and offset to grid center
      let cursor_pos = ( ( cursor_pos - canvas_pos ) - half_size ) / half_size / aspect_scale + grid_center;
      let selected_hex_coord : Coordinate< Axial, PointyTopped, OddParity > = layout.hex_coord( cursor_pos.into() );

      let path = find_path( Coordinate::new( 0, 0 ), selected_hex_coord, | coord | { map.borrow().map().get( &coord ).copied().unwrap_or_default() } );
      gl::info!( "{path:?}" );
      context.clear( gl::COLOR_BUFFER_BIT );

      // draw the grid
      hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
      hex_shader.uniform_upload( "u_color", &[ 0.7, 0.7, 0.7, 1.0 ] );
      filling_geometry.activate();
      context.draw_arrays( gl::TRIANGLES, 0, filling_geometry.nvertices );

      // draw obstacles
      for ( coord, obstacle ) in map.borrow().map()
      {
        if *obstacle
        {
          let hex_pos = layout.pixel_coord( *coord );
          let translation = mat2x2h::translate
          (
            [ hex_pos[ 0 ] - grid_center[ 0 ],
            -hex_pos[ 1 ] + grid_center[ 1 ] ]
          );
          let selected_mvp = scale_m * translation;

          hex_shader.uniform_matrix_upload( "u_mvp", selected_mvp.raw_slice(), true );
          hex_shader.uniform_upload( "u_color", &[ 0.7, 0.5, 0.5, 1.0 ] );
          context.draw_arrays( gl::TRIANGLES, 0, hex_geometry.nvertices );
        }
      }

      // draw the selected hexagon
      if map.borrow().map().keys().any( | k | *k == selected_hex_coord )
      {
        let selected_hex_pos = layout.pixel_coord( selected_hex_coord );
        let translation = mat2x2h::translate
        (
          [ selected_hex_pos[ 0 ] - grid_center[ 0 ],
          -selected_hex_pos[ 1 ] + grid_center[ 1 ] ]
        );
        let selected_mvp = scale_m * translation;

        hex_shader.uniform_matrix_upload( "u_mvp", selected_mvp.raw_slice(), true );
        hex_shader.uniform_upload( "u_color", &[ 0.5, 0.7, 0.5, 1.0 ] );
        context.draw_arrays( gl::TRIANGLES, 0, hex_geometry.nvertices );
      }

      // draw the outline
      hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
      hex_shader.uniform_upload( "u_color", &[ 0.1, 0.1, 0.1, 1.0 ] );
      outline_geometry.activate();
      context.draw_arrays( gl::LINES, 0, outline_geometry.nvertices );
    }
  };

  let mouse_move = Closure::< dyn Fn( _ ) >::new( Box::new( mouse_move ) );
  canvas.set_onmousemove( Some( mouse_move.as_ref().unchecked_ref() ) );
  mouse_move.forget();

  let mouse_click =
  {
    let map = map.clone();
    let canvas = canvas.clone();
    move | e : MouseEvent |
    {

      let rect = canvas.get_bounding_client_rect();
      let canvas_pos = F32x2::new( rect.left() as f32, rect.top() as f32 );
      let half_size : F32x2 = canvas_size / 2.0;
      let cursor_pos = F32x2::new( e.client_x() as f32, e.client_y() as f32 );
      // normalize coodinates to [ -1 : 1 ], then apply inverse ascpect scale and offset to grid center
      let cursor_pos = ( ( cursor_pos - canvas_pos ) - half_size ) / half_size / aspect_scale + grid_center;
      let selected_hex_coord : Coordinate< Axial, PointyTopped, OddParity > = layout.hex_coord( cursor_pos.into() );
      map.borrow_mut().map_mut().entry( selected_hex_coord ).and_modify( | v | *v = !*v );
    }
  };

  let mouse_click = Closure::< dyn Fn( _ ) >::new( Box::new( mouse_click ) );
  canvas.set_onmousedown( Some( mouse_click.as_ref().unchecked_ref() ) );
  mouse_click.forget();

  Ok( () )
}

// A* implementation for hexagonal grid
fn find_path< Orientation, Parity, F >
(
  start : Coordinate< Axial, Orientation, Parity >,
  end : Coordinate< Axial, Orientation, Parity >,
  is_accessible : F
)
-> Option< Vec< Coordinate< Axial, Orientation, Parity > > >
where
  Orientation : OrientationType,
  Parity : ParityType,
  F : Fn( Coordinate< Axial, Orientation, Parity > ) -> bool,
{
  // Node for A* algorithm

  // Define the six possible directions in axial coordinates
  let directions =
  [
    (  1,  0 ),  // East
    (  1, -1 ),  // Northeast
    (  0, -1 ),  // Northwest
    ( -1,  0 ),  // West
    ( -1,  1 ),  // Southwest
    (  0,  1 ),  // Southeast
  ];

  // Heuristic function: axial distance
  let heuristic =
  | a : &Coordinate< Axial, Orientation, Parity >, b : &Coordinate< Axial, Orientation, Parity > | -> i32
  {
    let ( q1, r1 ) = ( a.q, a.r );
    let ( q2, r2 ) = ( b.q, b.r );

    // Hex distance in axial coordinates
    let dq = q2 - q1;
    let dr = r2 - r1;
    let ds = -dq - dr; // Derived from axial coordinates: s = -q-r

    ( dq.abs() + dr.abs() + ds.abs() ) / 2
  };

  // The open set (priority queue)
  let mut open_set = std::collections::BinaryHeap::new();
  open_set.push
  (
    Node
    {
      coord : start,
      f_score : heuristic( &start, &end ),
      g_score : 0,
    }
  );

  // The set of visited nodes
  let mut came_from : HashMap< _, Coordinate< _, _, _ > > = HashMap::new();

  // Cost from start to node
  let mut g_score = HashMap::new();
  g_score.insert( start, 0 );

  while let Some( current_node ) = open_set.pop()
  {
    let current = current_node.coord;

    // If we've reached the goal
    if current == end
    {
      // Reconstruct path
      let mut path = vec![ end.clone() ];
      let mut current = end;

      while let Some( prev ) = came_from.get( &current )
      {
        path.push( prev.clone() );
        current = prev.clone();
      }

      path.reverse();
      return Some( path );
    }

      // For each neighbor
    for ( dq, dr ) in &directions
    {
      // Create neighbor coordinate
      let (q, r) = ( current.q, current.r );
      let neighbor = Coordinate::new( q + dq, r + dr );

      // Skip if not accessible
      if !is_accessible( neighbor.clone() ) { continue; }

      // Calculate tentative g score
      let mut tentative_g_score = *g_score.get( &current ).unwrap_or( &i32::MAX );
      if tentative_g_score < i32::MAX
      {
        tentative_g_score += 1;
      }

      // If this path to neighbor is better than any previous one, record it
      if tentative_g_score < *g_score.get( &neighbor ).unwrap_or( &i32::MAX )
      {
        came_from.insert( neighbor, current );
        g_score.insert( neighbor, tentative_g_score );

        let f_score = tentative_g_score + heuristic( &neighbor, &end );
        open_set.push
        (
          Node
          {
            coord : neighbor,
            f_score,
            g_score : tentative_g_score,
          }
        );
      }
    }
  }

  None
}

struct Node< Orientation, Parity >
where
  Orientation : OrientationType,
  Parity : ParityType,
{
  coord : Coordinate< Axial, Orientation, Parity >,
  f_score : i32,
  g_score : i32,
}

impl< Orientation, Parity > PartialEq for Node< Orientation, Parity >
where
  Orientation : OrientationType,
  Parity : ParityType,
{
  fn eq(&self, other: &Self) -> bool
  {
    self.coord == other.coord && self.f_score == other.f_score && self.g_score == other.g_score
  }
}

impl< Orientation, Parity > Eq for Node< Orientation, Parity >
where
  Orientation : OrientationType,
  Parity : ParityType,
{}

impl< Orientation, Parity > PartialOrd for Node< Orientation, Parity >
where
  Orientation : OrientationType,
  Parity : ParityType,
{
  fn partial_cmp( &self, other: &Self ) -> Option< std::cmp::Ordering >
  {
    // Reverse ordering for min-heap ( BinaryHeap is a max-heap by default )
    Some( other.f_score.cmp( &self.f_score ) )
  }
}

// We only compare nodes by their f_score for the priority queue
impl< Orientation, Parity > Ord for Node< Orientation, Parity >
where
  Orientation : OrientationType,
  Parity : ParityType,
{
  fn cmp(&self, other: &Self) -> std::cmp::Ordering
  {
    other.f_score.cmp( &self.f_score )
  }
}
