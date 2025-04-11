use tiles_tools::
{
  coordinates::{ hexagonal::*, pixel::Pixel },
  layout::*,
  geometry
};

use minwebgl as min;
use min::
{
  math::{ F32x2, IntoVector, mat2x2h },
  Program,
  JsCast,
  canvas::HtmlCanvasElement,
  GL,
  // web::log::info,
  // aaa : this import does not work, but not clear why
  // make it working please
  // it just does not work 😕
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

  // inclusize grid bounds
  let region = [ Coordinate::< Offset< Odd >, Pointy >::new( 0, 0 ), Coordinate::< Offset< _ >, _ >::new( 7, 5 ) ];
  // aaa : why shift_type is not part of layout? o.O
  // aaa : what about type Grid combinging layout and grid size. also grid probably can have offset of orign?
  let rect = RectangularGrid::new( region );
  let grid_center = rect.center();

  min::info!( "grid center: {grid_center:?}" );

  let grid_mesh = geometry::from_iter
  (
    rect.coordinates().map( | c | { Into::< Coordinate< Axial, Pointy > >::into( c ) } ),
    || geometry::hexagon_triangles(),
    mat2x2h::rot( 30.0f32.to_radians() ) * mat2x2h::scale( [ 0.9, 0.9 ] )
  );

  let aspect = canvas_size[ 1 ] / canvas_size[ 0 ];
  let scale = 0.12;
  let aspect_scale : F32x2 = [ aspect * scale, scale ].into();
  let scale_m = mat2x2h::scale( aspect_scale.0 );

  let vert = include_str!( "shaders/main.vert" );
  let frag = include_str!( "shaders/main.frag" );
  let hex_shader = Program::new( context.clone(), vert, frag )?;
  hex_shader.activate();

  let grid_geometry = min::geometry::Positions::new
  (
    context.clone(),
    &grid_mesh, // aaa : iterating all tiles several times is not efficient. is it possible to avoid it?
    2,
  )?;
  let outline_geometry = min::geometry::Positions::new
  (
    context.clone(),
    &geometry::hexagon_lines(),
    2,
  )?;

  let translation = mat2x2h::translate( [ -grid_center.x(), grid_center.y() ] );
  let mvp = scale_m * translation;

  context.clear_color( 0.9, 0.9, 0.9, 1.0 );
  context.clear( min::COLOR_BUFFER_BIT );
  hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
  hex_shader.uniform_upload( "u_color", &[ 0.7, 0.7, 0.7, 1.0 ] );
  grid_geometry.activate();
  context.draw_arrays( GL::TRIANGLES, 0, grid_geometry.nvertices );

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
      // aaa : where is center? in the middle? what are boundaries -1, +1? explain all that instead of duplicating what is avaliable from code
      // normalize coodinates to NDC [ -1 : 1 ], then apply inverse ascpect scale and offset to grid center
      // this transforms cursor position to the world space
      // then offset it by center of the grid, so that if cursor is in the center of the canvas, it will be in the center of the grid
      let cursor_pos : Pixel = ( ( ( cursor_pos - canvas_pos ) - half_size ) / ( half_size * aspect_scale ) + grid_center ).into(); // aaa : don't use double devission it's confusing and difficult to read. use canonical represenation

      let selected_hex_coord : Coordinate::< Axial, Pointy > = cursor_pos.into();

      if selected_hex.is_some_and( | hex_coord | hex_coord == selected_hex_coord )
      {
        return;
      }
      // aaa : add commented out code to see mouse position in log.
      // min::info!( "selected hex: {selected_hex_coord:?}" );

      selected_hex = Some( selected_hex_coord );

      context.clear( min::COLOR_BUFFER_BIT );

      // draw grid
      hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
      hex_shader.uniform_upload( "u_color", &[ 0.7, 0.7, 0.7, 1.0 ] );
      grid_geometry.activate();
      context.draw_arrays( min::TRIANGLES, 0, grid_geometry.nvertices );

      let selected_hex_pos : Pixel = selected_hex_coord.into();
      let translation = mat2x2h::translate( [ selected_hex_pos[ 0 ] - grid_center[ 0 ], -selected_hex_pos[ 1 ] + grid_center[ 1 ] ] );
      let mvp = scale_m * translation * mat2x2h::rot( 30.0f32.to_radians() );

      // draw outline
      hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
      hex_shader.uniform_upload( "u_color", &[ 0.1, 0.1, 0.1, 1.0 ] );
      outline_geometry.activate();
      context.draw_arrays( GL::LINES, 0, outline_geometry.nvertices ); // aaa : don't use loop geometry, it has limmited suport among backends
                                                                       // i added default lines mesh generation support, but for this webgl rendering i think line loop is okay
                                                                       // aaa : let's use linestrip. rid of loops
    }
  };

  let mouse_move = Closure::< dyn FnMut( _ ) >::new( Box::new( mouse_move ) );
  canvas.set_onmousemove( Some( mouse_move.as_ref().unchecked_ref() ) );
  mouse_move.forget();

  Ok( () )
}
