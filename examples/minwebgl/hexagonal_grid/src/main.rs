// pub mod webgl_render;

use tiles_tools::
{
  layout::{ Orientation, HexLayout },
  coordinates::{ Pixel, Coordinate, Axial },
  mesh::{ hex_line_loop_mesh, hex_triangle_fan_mesh },
  patterns::{ Parity, ShiftedRectangleIter },
};

use minwebgl as gl;
use gl::
{
  GL,
  math::{ d2::mat2x2h, U32x2, F64x2 },
  Program,
  JsCast,
  canvas::HtmlCanvasElement,
  geometry,
  // web::log::info,
  // qqq : this import does not work, but not clear why
  // make it working please
};

use web_sys::{ wasm_bindgen::prelude::Closure, MouseEvent };
// use webgl_render::HexShader;

fn main() -> Result< (), gl::WebglError >
{
  draw_hexes()
}

fn draw_hexes() -> Result< (), minwebgl::WebglError >
{
  gl::browser::setup( Default::default() );
  let context = gl::context::retrieve_or_make_reduced_dpr()?;
  // qqq : Instead of this function, please introduce the function `retrieve_or_make_with( o )` where `o` is a structure containing options and a builder for them.
  // qqq : add to structure Options other relevant options of retreiving context

  let canvas = context.canvas().unwrap().dyn_into::< HtmlCanvasElement >().unwrap();

  // qqq : use vector or tuple
  let canvas_size : U32x2 = ( canvas.width(), canvas.height() ).into_vector();

  // qqq : explain why does it required
  let dpr = web_sys::window().unwrap().device_pixel_ratio();
  // gl::log::info!( "dpr : {:#?}", dpr );
  // gl::web::log::info!( "dpr : {:#?}", dpr );

  context.clear_color( 0.9, 0.9, 0.9, 1.0 );

  // qqq : collect all parameters into a single block of code
  // qqq : what are units? not clear
  // size of a hexagon (from center to vertex)
  let size = 0.1;

  // how to shift the hexagons to form a rectangle
  let shift_type = Parity::Even;
  // orientation of hex can be either pointing upword or flat upword
  let orientation = Orientation::Flat;

  // orientation of the hexagons
  let layout = HexLayout { orientation, size };
  // grid size
  let grid_size = [ 9, 11 ];
  // let rows = 9;
  // let columns = 11;
  // determine the center of the grid
  // to shift it to the center of the canvas
  // qqq : use vector or tuple
  // let ( center_x, center_y ) = layout.grid_center( ShiftedRectangleIter::new( grid_size, shift_type, layout ) );
  let grid_center : F64x2 = layout.grid_center( ShiftedRectangleIter::new( grid_size, shift_type, layout ) ).into();

  // let hex_shader = HexShader::new( &context )?;
  let vert = include_str!( "shaders/main.vert" );
  let frag = include_str!( "shaders/main.frag" );
  let hex_shader = Program::new( context.clone(), vert, frag )?;
  hex_shader.activate();
  // triangular fan mesh for of a hexagon
  let triangle_geometry = geometry::Positions::new( context.clone(), &hex_triangle_fan_mesh( &layout ), 2 )?;
  // line loop mesh for the outline of a hexagon
  let line_geometry = geometry::Positions::new( context.clone(), &hex_line_loop_mesh( &layout ), 2 )?;

  let aspect = canvas_size[ 1 ] as f32 / canvas_size[ 0 ] as f32;
  let scale = 1.0;
  let aspect_scale = [ aspect * scale, 1.0 * scale ];
  let scale_m = mat2x2h::scale( aspect_scale );

  let mut selected_hex = None;

  let mouse_move =
  {
    let context = context.clone();
    let canvas = canvas.clone();
    move | e : MouseEvent |
    {
      let rect = canvas.get_bounding_client_rect();
      let canvas_pos : F64x2 = F64x2::new( rect.left(), rect.top() ).try_into().unwrap();
      // let canvas_x = rect.left();
      // let canvas_y = rect.top();

      // transform mouse coordinates from pixels to world coordinates
      // where the center of the canvas is ( 0.0, 0.0 )
      // qqq : use vector
      let canvas_half_size : F64x2 = 0.5 * canvas_size.into() / dpr;
      // let half_width = ( 0.5 * width as f64 / dpr ) as f32;
      // let half_height = ( 0.5 * height as f64 / dpr ) as f32;

      let mouse_pos = F64x2::new( e.client_x(), e.client_y() ) - canvas_pos;
      // let x = ( e.client_x() as f64 - canvas_x ) as f32;
      // let y = ( e.client_y() as f64 - canvas_y ) as f32;
      // qqq : buy why you do that? name all coordinates
      // normalize then multiply by inverse aspect_scale
      // and offset by center of the grid
      // let x = ( x - half_width ) / half_width * ( 1.0 / aspect_scale[ 0 ] ) + center_x;
      // let y = ( y - half_height ) / half_height * ( 1.0 / aspect_scale[ 1 ] ) + center_y;
      let mouse_pos_nameme = grid_center + ( mouse_pos - canvas_half_size ) / ( canvas_half_size * aspect_scale );

      // qqq : put bounds on parameters so that it was not possible to pass () as parameter value
      let cursor_coord : Coordinate< Axial, (), () > = layout.hex_coord( mouse_pos_nameme );

      // qqq : currently it's borken and don't draw grid until mouse move
      // rerender only if the selected hexagon has changed
      if selected_hex.is_some_and( | hex | hex == cursor_coord )
      {
        return;
      }

      selected_hex = Some( cursor_coord );

      context.clear( gl::COLOR_BUFFER_BIT );

      // draw outline
      // hexagon center in world coords
      let pixel_coord = layout.pixel_coord( cursor_coord );
      // offset by center of the grid
      let translation = mat2x2h::translate( [ pixel_coord[ 0 ] - grid_center[ 0 ], -pixel_coord[ 1 ] + grid_center[ 1 ] ] );
      // let scale = mat2x2h::scale( [ size, size ] );
      let mvp = scale_m * translation;

      // qqq : too many draw calls!
      // draw hexes
      for coord in ShiftedRectangleIter::new( grid_size, shift_type, layout )
      {
        // hexagon center in world coords
        let pixel_coord = layout.pixel_coord( coord );

        let position = [ pixel_coord[ 0 ] - grid_center[ 0 ], -pixel_coord[ 1 ] + grid_center[ 1 ] ];
        let translation = mat2x2h::translate( position );
        let scale = mat2x2h::scale( [ 0.95, 0.95 ] );
        let mvp = scale_m * translation * scale;
        hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
        hex_shader.uniform_upload( "u_color", &[ 0.3, 0.3, 0.3, 1.0 ] );
        triangle_geometry.activate();
        context.draw_arrays( GL::TRIANGLE_FAN, 0, triangle_geometry.nvertices );
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
      }

      hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
      hex_shader.uniform_upload( "u_color", &[ 0.1, 0.9, 0.1, 1.0 ] );
      line_geometry.activate();
      context.draw_arrays( GL::LINE_LOOP, 0, line_geometry.nvertices );
      // hex_shader.draw( gl::LINE_LOOP, &line_geometry ).unwrap();
      // hex_shader.draw( &context, gl::LINE_LOOP, &line_geometry, mvp.raw_slice(), [ 0.3, 0.3, 0.3, 1.0 ] ).unwrap();

    }
  };
  let mouse_move = Closure::< dyn FnMut( _ ) >::new( Box::new( mouse_move ) );
  canvas.set_onmousemove( Some( mouse_move.as_ref().unchecked_ref() ) );
  mouse_move.forget();

  Ok( () )
}
