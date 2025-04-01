use tiles_tools::
{
  coordinates::*,
  layout::{ HexLayout, Orientation },
  mesh::{ grid_triangle_mesh, hex_line_loop_mesh },
  patterns::{ Parity, ShiftedRectangleIter }
};

use minwebgl as gl;
use gl::
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

fn main() -> Result< (), gl::WebglError >
{
  draw_hexes()
}

fn draw_hexes() -> Result< (), minwebgl::WebglError >
{
  // qqq : Instead of this function, please introduce the function
  // `retrieve_or_make_with( o )` where `o` is a structure containing options and a builder for them.
  // qqq : add to structure Options other relevant options of retreiving context
  gl::browser::setup( Default::default() );
  let context = gl::context::retrieve_or_make_with( gl::context::ReducedDprBuilder )?;
  let canvas = context.canvas().unwrap().dyn_into::< HtmlCanvasElement >().unwrap();
  // qqq : explain why does it required
  // used to scale canvas true size to css size
  let dpr = web_sys::window().unwrap().device_pixel_ratio() as f32;
  // qqq : use vector or tuple
  let canvas_size = ( canvas.width() as f32, canvas.height() as f32 ).into_vector() / dpr;
  // gl::log::info!( "dpr : {:#?}", dpr );
  // gl::web::log::info!( "dpr : {:#?}", dpr );

  // qqq : collect all parameters into a single block of code
  // qqq : what are units? not clear
  // just abstract size in world space, it may be any units
  // size of a hexagon ( from center to vertex )
  let size = 0.1;
  // how to shift the hexagons to form a rectangle
  let shift_type = Parity::Odd;
  // orientation of hex can be either pointing upword or flat upword
  let orientation = Orientation::Pointy;
  // orientation of the hexagons
  let layout = HexLayout { orientation, size };
  // grid size
  let grid_size = [ 9, 11 ];
  // determine the center of the grid
  // to shift it to the center of the canvas
  // qqq : use vector or tuple
  let grid_center : F32x2 = layout.grid_center( ShiftedRectangleIter::new( grid_size, shift_type, layout ) ).into();

  let vert = include_str!( "shaders/main.vert" );
  let frag = include_str!( "shaders/main.frag" );
  let hex_shader = Program::new( context.clone(), vert, frag )?;
  hex_shader.activate();

  let grid_geometry = geometry::Positions::new
  (
    context.clone(),
    &grid_triangle_mesh( ShiftedRectangleIter::new( grid_size, shift_type, layout ), &layout, None ),
    2
  )?;
  // line loop mesh for the outline of a hexagon
  let outline_geometry = geometry::Positions::new( context.clone(), &hex_line_loop_mesh( &layout ), 2 )?;

  let aspect = canvas_size[ 1 ] / canvas_size[ 0 ];
  let scale = 1.0;
  let aspect_scale : F32x2 = [ aspect * scale, scale ].into();
  let scale_m = mat2x2h::scale( aspect_scale.0 );

  let translation = mat2x2h::translate( [ -grid_center.x(), grid_center.y() ] );
  let mvp = scale_m * translation;

  context.clear_color( 0.9, 0.9, 0.9, 1.0 );
  context.clear( gl::COLOR_BUFFER_BIT );
  hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
  hex_shader.uniform_upload( "u_color", &[ 0.1, 0.1, 0.1, 1.0 ] );
  grid_geometry.activate();
  context.draw_arrays( gl::TRIANGLES, 0, grid_geometry.nvertices );

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
      let cursor_pos = ( ( cursor_pos - canvas_pos ) - half_size ) / half_size / aspect_scale + grid_center;
      let selected_hex_coord : Coordinate< Axial, PointyTopped, OddParity > = layout.hex_coord( cursor_pos.into() );

      if selected_hex.is_some_and( | hex_coord | hex_coord == selected_hex_coord )
      {
        return;
      }
      selected_hex = Some( selected_hex_coord );

      context.clear( gl::COLOR_BUFFER_BIT );

      hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
      hex_shader.uniform_upload( "u_color", &[ 0.1, 0.1, 0.1, 1.0 ] );
      grid_geometry.activate();
      context.draw_arrays( gl::TRIANGLES, 0, grid_geometry.nvertices );

      let selected_hex_pos : F32x2 = layout.pixel_coord( selected_hex_coord ).data.into();
      let translation = mat2x2h::translate( [ selected_hex_pos[ 0 ] - grid_center[ 0 ], -selected_hex_pos[ 1 ] + grid_center[ 1 ] ] );
      let mvp = scale_m * translation;

      hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
      hex_shader.uniform_upload( "u_color", &[ 0.1, 0.9, 0.1, 1.0 ] );
      outline_geometry.activate();
      context.draw_arrays( GL::LINE_LOOP, 0, outline_geometry.nvertices );

      // gl::info!( "{:#?}", cursor_pos );
      // transform mouse coordinates from pixels to world coordinates
      // where the center of the canvas is ( 0.0, 0.0 )
      // qqq : use vector
      // let canvas_half_size : F32x2 = 0.5 * canvas_size.into() / dpr;
      // let half_width = ( 0.5 * width as f32 / dpr ) as f32;
      // let half_height = ( 0.5 * height as f32 / dpr ) as f32;

      // let mouse_pos = F32x2::new( e.client_x().into(), e.client_y().into() ) - canvas_pos;
      // let x = ( e.client_x() as f32 - canvas_x ) as f32;
      // let y = ( e.client_y() as f32 - canvas_y ) as f32;
      // qqq : buy why you do that? name all coordinates
      // normalize then multiply by inverse aspect_scale
      // and offset by center of the grid
      // let x = ( x - half_width ) / half_width * ( 1.0 / aspect_scale[ 0 ] ) + center_x;
      // let y = ( y - half_height ) / half_height * ( 1.0 / aspect_scale[ 1 ] ) + center_y;
      // let aspect_scale : F64x2 =  [ aspect_scale[ 0 ].into(), aspect_scale[ 1 ].into() ].into();
      // let grid_center : F64x2 = [ grid_center[ 0 ].into(), grid_center[ 1 ].into() ].into();
      // let mouse_pos: F64x2 = ( ( mouse_pos - canvas_half_size ) / canvas_half_size / aspect_scale ) + grid_center;

      // qqq : put bounds on parameters so that it was not possible to pass () as parameter value


      // qqq : currently it's borken and don't draw grid until mouse move
      // fixed
      // rerender only if the selected hexagon has changed

      // qqq : too many draw calls!
      // draw hexes
      // let translation = mat2x2h::translate( [ -center.0, center.1 ] );
      // let mvp = scale_m * translation;
      // hex_shader.draw
      // (
      //   &context,
      //   gl::TRIANGLES, // qqq : avoid using fan, it's too specific mesh primitive type
      //   &grid_mesh,
      //   mvp.raw_slice(),
      //   [ 0.3, 0.75, 0.3, 1.0 ].as_slice(), // qqq : parametrize
      // ).unwrap();

      // draw outline
      // hexagon center in world coords
      // let pixel_coord = layout.pixel_coord( cursor_coord );
      // offset by center of the grid

      // let translation = mat2x2h::translate( [ x - grid_center.x(), -y + grid_center.y() ] );
      // let scale = mat2x2h::scale( [ size, size ] );
      // let mvp = scale_m * translation;
      // hex_shader.draw( &context, gl::LINE_LOOP, &line_geometry, mvp.raw_slice(), [ 0.3, 0.3, 0.3, 1.0 ].as_slice() ).unwrap();

      // let translation = mat2x2h::translate( [ pixel_coord[ 0 ] - grid_center[ 0 ], -pixel_coord[ 1 ] + grid_center[ 1 ] ] );
      // let scale = mat2x2h::scale( [ size, size ] );
      // let mvp = scale_m * translation;

      // qqq : too many draw calls!
      // draw hexes
      // for coord in ShiftedRectangleIter::new( grid_size, shift_type, layout )
      // {
        // hexagon center in world coords
        // let pixel_coord = layout.pixel_coord( coord );

        // let position = [ pixel_coord[ 0 ] - grid_center[ 0 ], -pixel_coord[ 1 ] + grid_center[ 1 ] ];
        // let translation = mat2x2h::translate( position );
        // let scale = mat2x2h::scale( [ 0.95, 0.95 ] );
        // let mvp = scale_m * translation * scale;
        // hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
        // hex_shader.uniform_upload( "u_color", &[ 0.3, 0.3, 0.3, 1.0 ] );
        // triangle_geometry.activate();
        // context.draw_arrays( GL::TRIANGLE_FAN, 0, triangle_geometry.nvertices );


        // gl::log::info!( "triangle_geometry.nvertices : {}", triangle_geometry.nvertices );
        // context.draw_arrays( GL::TRIANGLE_FAN, 0, 6 );
        // qqq : avoid using fan, it's too specific mesh primitive type
        // hex_shader.draw
        // (
        //   &context,
        //   gl::TRIANGLE_FAN,
        //   &triangle_geometry,
        //   mvp.raw_slice(),
        //   [ 0.3, 0.75, 0.3, 1.0 ], // qqq : parametrize
        // ).unwrap();
      // }
      // hex_shader.draw( gl::LINE_LOOP, &line_geometry ).unwrap();
      // hex_shader.draw( &context, gl::LINE_LOOP, &line_geometry, mvp.raw_slice(), [ 0.3, 0.3, 0.3, 1.0 ] ).unwrap();

    }
  };
  let mouse_move = Closure::< dyn FnMut( _ ) >::new( Box::new( mouse_move ) );
  canvas.set_onmousemove( Some( mouse_move.as_ref().unchecked_ref() ) );
  mouse_move.forget();

  Ok( () )
}
