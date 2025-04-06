use tiles_tools::
{
  coordinates::*,
  layout::{ HexLayout, Orientation },
  mesh::{ grid_triangle_mesh, hex_line_loop_mesh }, // qqq : don't import from namespace individualt items use full names for such cases `mesh::grid_triangle` remove postfix _mesh
  patterns::{ Parity, ShiftedRectangleIter }
};

use minwebgl as min;
use min::
{
  GL,
  math::{ d2::mat2x2h, F32x2, IntoVector },
  Program,
  JsCast,
  canvas::HtmlCanvasElement,
  geometry,
  // web::log::info,
  // qqq : this import does not work, but not clear why
  // make it working please
};
use web_sys::{ wasm_bindgen::prelude::Closure, MouseEvent };

fn main() -> Result< (), min::WebglError >
{
  draw_hexes()
}

fn draw_hexes() -> Result< (), minwebgl::WebglError >
{
  min::browser::setup( Default::default() );
  let o = min::context::ContexOptions::new().reduce_dpr( true );
  let context = min::context::retrieve_or_make_with( o )?;
  let canvas = context.canvas().unwrap().dyn_into::< HtmlCanvasElement >().unwrap();
  // used to scale canvas true size to css size
  let dpr = web_sys::window().unwrap().device_pixel_ratio() as f32;
  let canvas_size = ( canvas.width() as f32, canvas.height() as f32 ).into_vector() / dpr;
  // min::log::info!( "dpr : {:#?}", dpr );
  // min::web::log::info!( "dpr : {:#?}", dpr );

  // just abstract size in world space, it may be any units
  // size of a hexagon ( from center to vertex )
  let hex_size = 0.1;
  // how to shift the hexagons to form a rectangle
  let shift_type = Parity::Odd; // qqq : why sift type is not part of layout? it probably should be
  // orientation of hex can be either pointing upword or flat upword
  let orientation = Orientation::Pointy;
  // orientation of the hexagons
  let layout = HexLayout { orientation, size: hex_size }; // aaa : size of what specifically? not clear
  // grid size
  let grid_size = [ 9, 11 ];

  // determine the center of the grid
  // to shift it to the center of the canvas
  // qqq : what about type Grid combinging layout and grid size. also grid probably can have offset of orign?
  let grid_center : F32x2 = layout.grid_center( ShiftedRectangleIter::new( grid_size, shift_type, layout ) ).into(); // qqq : iterating all tiles several times is not efficient. is it possible to avoid it?
  // qqq : why shift_type is not part of layout? o.O
  let aspect = canvas_size[ 1 ] / canvas_size[ 0 ];
  let scale = 1.0;
  let aspect_scale : F32x2 = [ aspect * scale, scale ].into();
  let scale_m = mat2x2h::scale( aspect_scale.0 );

  let vert = include_str!( "shaders/main.vert" );
  let frag = include_str!( "shaders/main.frag" );
  let hex_shader = Program::new( context.clone(), vert, frag )?;
  hex_shader.activate();

  let grid_geometry = geometry::Positions::new
  (
    context.clone(),
    &grid_triangle_mesh( ShiftedRectangleIter::new( grid_size, shift_type, layout ), &layout, None ), // qqq : iterating all tiles several times is not efficient. is it possible to avoid it?
    2,
  )?;
  // line loop mesh for the outline of a hexagon
  let outline_geometry = geometry::Positions::new
  (
    context.clone(),
    &hex_line_loop_mesh( &layout ),
    2,
  )?;

  let translation = mat2x2h::translate( [ -grid_center.x(), grid_center.y() ] );
  let mvp = scale_m * translation;

  context.clear_color( 0.9, 0.9, 0.9, 1.0 );
  context.clear( min::COLOR_BUFFER_BIT );
  hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
  hex_shader.uniform_upload( "u_color", &[ 0.1, 0.1, 0.1, 1.0 ] );
  grid_geometry.activate();
  context.draw_arrays( min::TRIANGLES, 0, grid_geometry.nvertices );

  let mut selected_hex = None;

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
      let cursor_pos = ( ( cursor_pos - canvas_pos ) - half_size ) / half_size / aspect_scale + grid_center; // qqq : don't use double devission it's confusing and difficult to read. use canonical represenation
      // qqq : add commented out code to see it in log.
      // qqq : where is center? in the middle? what are boundaries -1, +1? explain all that instead of duplicating what is avaliable from code
      let selected_hex_coord : Coordinate< Axial, PointyTopped, OddParity > = layout.hex_coord( cursor_pos.into() );

      if selected_hex.is_some_and( | hex_coord | hex_coord == selected_hex_coord )
      {
        return;
      }
      selected_hex = Some( selected_hex_coord );

      context.clear( min::COLOR_BUFFER_BIT );

      // draw grid
      hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
      hex_shader.uniform_upload( "u_color", &[ 0.1, 0.1, 0.1, 1.0 ] );
      grid_geometry.activate();
      context.draw_arrays( min::TRIANGLES, 0, grid_geometry.nvertices );

      let selected_hex_pos = layout.pixel_coord( selected_hex_coord );
      let translation = mat2x2h::translate( [ selected_hex_pos[ 0 ] - grid_center[ 0 ], -selected_hex_pos[ 1 ] + grid_center[ 1 ] ] );
      let mvp = scale_m * translation;

      // draw outline
      hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
      hex_shader.uniform_upload( "u_color", &[ 0.1, 0.9, 0.1, 1.0 ] );
      outline_geometry.activate();
      context.draw_arrays( GL::LINE_LOOP, 0, outline_geometry.nvertices ); // aaa : don't use loop geometry, it has limmited suport among backends
                                                                           // i added default lines mesh generation support, but for this webgl rendering i think line loop is okay
                                                                           // qqq : let's use linestrip
    }
  };

  let mouse_move = Closure::< dyn FnMut( _ ) >::new( Box::new( mouse_move ) );
  canvas.set_onmousemove( Some( mouse_move.as_ref().unchecked_ref() ) );
  mouse_move.forget();

  Ok( () )
}
