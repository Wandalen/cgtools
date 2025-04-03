use tiles_tools::
{
  coordinates::*,
  layout::{ HexLayout, Orientation },
  mesh,
  patterns::{ Parity, ShiftedRectangleIter }
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
  draw_hexes()
}

fn draw_hexes() -> Result< (), minwebgl::WebglError >
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

  let mouse_move =
  {
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

      context.clear( gl::COLOR_BUFFER_BIT );

      hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
      hex_shader.uniform_upload( "u_color", &[ 0.7, 0.7, 0.7, 1.0 ] );
      filling_geometry.activate();
      context.draw_arrays( gl::TRIANGLES, 0, filling_geometry.nvertices );

      for ( coord, obstacle ) in map.map()
      {
        if *obstacle
        {
          let hex_pos = layout.pixel_coord( *coord );
          let translation = mat2x2h::translate( [ hex_pos[ 0 ] - grid_center[ 0 ], -hex_pos[ 1 ] + grid_center[ 1 ] ] );
          let selected_mvp = scale_m * translation;

          hex_shader.uniform_matrix_upload( "u_mvp", selected_mvp.raw_slice(), true );
          hex_shader.uniform_upload( "u_color", &[ 0.7, 0.5, 0.5, 1.0 ] );
          context.draw_arrays( gl::TRIANGLES, 0, hex_geometry.nvertices );
        }
      }

      if map.map().keys().any( | k | *k == selected_hex_coord )
      {
        let selected_hex_pos = layout.pixel_coord( selected_hex_coord );
        let translation = mat2x2h::translate( [ selected_hex_pos[ 0 ] - grid_center[ 0 ], -selected_hex_pos[ 1 ] + grid_center[ 1 ] ] );
        let selected_mvp = scale_m * translation;

        hex_shader.uniform_matrix_upload( "u_mvp", selected_mvp.raw_slice(), true );
        hex_shader.uniform_upload( "u_color", &[ 0.5, 0.7, 0.5, 1.0 ] );
        context.draw_arrays( gl::TRIANGLES, 0, hex_geometry.nvertices );
      }

      hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
      hex_shader.uniform_upload( "u_color", &[ 0.1, 0.1, 0.1, 1.0 ] );
      outline_geometry.activate();
      context.draw_arrays( gl::LINES, 0, outline_geometry.nvertices );
    }
  };

  let mouse_move = Closure::< dyn FnMut( _ ) >::new( Box::new( mouse_move ) );
  canvas.set_onmousemove( Some( mouse_move.as_ref().unchecked_ref() ) );
  mouse_move.forget();

  Ok( () )
}
