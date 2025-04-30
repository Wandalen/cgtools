pub mod pathfind;
pub mod input;

use input::
{
  Action,
  Event,
  EventFlags,
  EventType,
  MouseButton
};
use tiles_tools::
{
  collection::HexArray,
  coordinates::{ hexagonal::*, pixel::Pixel },
  geometry,
  layout::*
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
use web_sys::{ wasm_bindgen::prelude::Closure, HtmlButtonElement, HtmlInputElement };
use std::{ cell::RefCell, collections::HashMap, rc::Rc };

fn main() -> Result< (), min::WebglError >
{
  draw_hexes()
}

fn draw_hexes() -> Result< (), minwebgl::WebglError >
{
  min::browser::setup( Default::default() );
  let o = min::context::ContexOptions::default()
  .remove_dpr_scaling( true )
  .preserve_drawing_buffer( true )
  .power_preference( minwebgl::context::PowerPreference::HighPerformance );
  let context = min::context::retrieve_or_make_with( o )?;
  let canvas = context.canvas().unwrap().dyn_into::< HtmlCanvasElement >().unwrap();
  // used to scale canvas true size to css size
  let dpr = web_sys::window().unwrap().device_pixel_ratio() as f32;
  let canvas_size = ( canvas.width() as f32, canvas.height() as f32 ).into_vector() / dpr;

  // inclusize grid bounds
  let region = [ Coordinate::< Offset< Odd >, Pointy >::new( 0, 0 ), Coordinate::< Offset< _ >, _ >::new( 8, 8 ) ];
  // aaa : why shift_type is not part of layout? o.O
  // aaa : what about type Grid combinging layout and grid size. also grid probably can have offset of orign?
  let rect = RectangularGrid::new( region );
  // coordinates of a point in the center of grid bounds
  let grid_center = rect.center();

  let grid_mesh = geometry::from_iter
  (
    rect.coordinates().map( | c | Into::< Coordinate< Axial, _ > >::into( c ) ),
    || geometry::hexagon_triangles(),
    mat2x2h::rot( 30.0f32.to_radians() ) * mat2x2h::scale( [ 0.9, 0.9 ] )
  );

  let aspect = canvas_size[ 1 ] / canvas_size[ 0 ];
  let scale = 0.1;
  let aspect_scale : F32x2 = [ aspect * scale, scale ].into();
  let scale_m = mat2x2h::scale( aspect_scale.0 );

  let vert = include_str!( "shaders/main.vert" );
  let frag = include_str!( "shaders/main.frag" );
  let hex_shader = Rc::new( Program::new( context.clone(), vert, frag )? );
  hex_shader.activate();

  let grid_geometry = Rc::new
  (
    min::geometry::Positions::new
    (
      context.clone(),
      &grid_mesh, // aaa : iterating all tiles several times is not efficient. is it possible to avoid it?
      2,
    )?
  );
  let outline_geometry = Rc::new
  (
    min::geometry::Positions::new
    (
      context.clone(),
      &geometry::hexagon_lines(),
      2,
    )?
  );
  let hexagon_geometry = Rc::new
  (
    min::geometry::Positions::new
    (
      context.clone(),
      &geometry::hexagon_triangles(),
      2,
    )?
  );

  let translation = mat2x2h::translate( [ -grid_center.x(), grid_center.y() ] );
  let mvp = scale_m * translation;

  let input = Rc::new( RefCell::new( input::Input::new( false, Some( canvas.clone().dyn_into().unwrap() ) ) ) );

  let document = web_sys::window().unwrap().document().unwrap();

  let grid_button : HtmlButtonElement = document
  .get_element_by_id( "grid" )
  .unwrap()
  .dyn_into()
  .unwrap();

  let pathfind_button : HtmlButtonElement = document
  .get_element_by_id( "pathfinding" )
  .unwrap()
  .dyn_into()
  .unwrap();

  let painting_button : HtmlButtonElement = document
  .get_element_by_id( "painting" )
  .unwrap()
  .dyn_into()
  .unwrap();


  ////// GRID DEMO //////

  let closure =
  {
    let input = input.clone();

    let context = context.clone();
    let canvas = canvas.clone();
    let grid_geometry = grid_geometry.clone();
    let hex_shader = hex_shader.clone();

    let mut selected_hex = None;

    move ||
    {
      let context = context.clone();
      let canvas = canvas.clone();
      let grid_geometry = grid_geometry.clone();
      let hex_shader = hex_shader.clone();
      let outline_geometry = outline_geometry.clone();

      hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
      hex_shader.uniform_upload( "u_color", &[ 0.7, 0.7, 0.7, 1.0 ] );
      grid_geometry.activate();
      context.draw_arrays( GL::TRIANGLES, 0, grid_geometry.nvertices );

      input.borrow_mut().clear_callbacks();

      let closure = move | input : &input::InputState, _ |
      {
        let rect = canvas.get_bounding_client_rect();
        let canvas_pos = F32x2::new( rect.left() as f32, rect.top() as f32 );
        let half_size : F32x2 = canvas_size / 2.0;
        let cursor_pos = F32x2::new( input.mouse_position()[ 0 ] as f32, input.mouse_position()[ 1 ] as f32 );
        // aaa : where is center? in the middle? what are boundaries -1, +1? explain all that instead of duplicating what is avaliable from code
        // normalize coodinates to NDC [ -1 : 1 ], then apply inverse ascpect scale and offset to grid center
        // this transforms cursor position to the world space
        // then offset it by center of the grid, so that if cursor is in the center of the canvas, it will be in the center of the grid
        let cursor_pos : Pixel =
        (
          ( ( cursor_pos - canvas_pos ) - half_size ) / ( half_size * aspect_scale ) + grid_center
        ).into(); // aaa : don't use double devission it's confusing and difficult to read. use canonical represenation

        let selected_hex_coord : Coordinate::< Axial, Pointy > = cursor_pos.into();

        if selected_hex.is_some_and( | hex_coord | hex_coord == selected_hex_coord )
        {
          return;
        }
        // aaa : add commented out code to see mouse position in log.
        // min::info!( "mouse pos: {cursor_pos:?}" );

        selected_hex = Some( selected_hex_coord );

        context.clear( GL::COLOR_BUFFER_BIT );

        // draw grid
        hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
        hex_shader.uniform_upload( "u_color", &[ 0.7, 0.7, 0.7, 1.0 ] );
        grid_geometry.activate();
        context.draw_arrays( GL::TRIANGLES, 0, grid_geometry.nvertices );

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
      };

      input.borrow_mut().add_callback( closure, EventFlags::MouseMovement );
    }
  };

  let onclick = Closure::< dyn FnMut() >::new( closure );
  grid_button.set_onclick( Some( onclick.as_ref().unchecked_ref() ) );
  onclick.forget();


  ////// PATHFIND DEMO //////

  let closure =
  {
    let input = input.clone();

    let canvas = canvas.clone();
    let context = context.clone();
    let hex_shader = hex_shader.clone();
    let grid_geometry = grid_geometry.clone();
    let hexagon_geometry = hexagon_geometry.clone();

    move ||
    {
      input.borrow_mut().clear_callbacks();

      context.clear( GL::COLOR_BUFFER_BIT );

      let canvas = canvas.clone();
      let context = context.clone();
      let hex_shader = hex_shader.clone();
      let grid_geometry = grid_geometry.clone();
      let hexagon_geometry = hexagon_geometry.clone();

      let mut start = Coordinate::< Axial, _ >::new( 2, 4 );
      let mut obstacles = HashMap::< Coordinate< Axial, Pointy >, bool >::from_iter
      (
        rect.coordinates().map( | c | ( c.into(), true ) )
      );

      let closure = move | input : &input::InputState, event |
      {
        let Event { event_type, .. } = event;

        let rect = canvas.get_bounding_client_rect();
        let canvas_pos = F32x2::new( rect.left() as f32, rect.top() as f32 );
        let half_size : F32x2 = canvas_size / 2.0;

        let [ x, y ] = input.mouse_position().0;
        let cursor_pos = F32x2::new( x as f32, y as f32 );
        let cursor_pos : Pixel =
        (
          ( ( cursor_pos - canvas_pos ) - half_size ) / ( half_size * aspect_scale ) + grid_center
        ).into();
        let selected_hex_coord : Coordinate::< Axial, Pointy > = cursor_pos.into();

        if let EventType::MouseButton( button, Action::Press ) = event_type
        {
          if button == MouseButton::Main
          && obstacles.contains_key( &selected_hex_coord )
          && selected_hex_coord != start
          {
            obstacles.entry( selected_hex_coord ).and_modify( | v | *v = !*v );
          }
          if button == MouseButton::Auxiliary
          && obstacles.get( &selected_hex_coord ).copied().unwrap_or_default()
          {
            start = selected_hex_coord;
          }
        }

        context.clear( GL::COLOR_BUFFER_BIT );

        // draw grid
        hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
        hex_shader.uniform_upload( "u_color", &[ 0.7, 0.7, 0.7, 1.0 ] );
        grid_geometry.activate();
        context.draw_arrays( GL::TRIANGLES, 0, grid_geometry.nvertices );

        // draw obstacles
        for ( &coord, _ ) in obstacles.iter().filter( | ( _, v ) | !**v )
        {
          let hex_pos : Pixel = coord.into();
          let translation = mat2x2h::translate
          (
            [ hex_pos[ 0 ] - grid_center[ 0 ], -hex_pos[ 1 ] + grid_center[ 1 ] ]
          );
          let mvp = scale_m * translation * mat2x2h::rot( 30.0f32.to_radians() ); // * mat2x2h::scale( [ 0.9, 0.9 ] );

          hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
          hex_shader.uniform_upload( "u_color", &[ 0.1, 0.1, 0.1, 1.0 ] );
          hexagon_geometry.activate();
          context.draw_arrays( GL::TRIANGLES, 0, hexagon_geometry.nvertices );
        }

        let goal = selected_hex_coord;

        let path = pathfind::find_path
        (
          &start,
          &goal,
          | coord | obstacles.get( &coord ).copied().unwrap_or_default()
        );

        if let Some( ( path, _ ) ) = path
        {
          // draw path
          for coord in path
          {
            let hex_pos : Pixel = coord.into();
            let translation = mat2x2h::translate
            (
              [ hex_pos[ 0 ] - grid_center[ 0 ], -hex_pos[ 1 ] + grid_center[ 1 ] ]
            );
            let mvp = scale_m * translation * mat2x2h::rot( 30.0f32.to_radians() ); // * mat2x2h::scale( [ 0.9, 0.9 ] );

            hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
            hex_shader.uniform_upload( "u_color", &[ 0.1, 0.6, 0.1, 1.0 ] );
            hexagon_geometry.activate();
            context.draw_arrays( GL::TRIANGLES, 0, hexagon_geometry.nvertices );
          }
        }
      };

      input.borrow_mut().add_callback
      (
        closure,
        EventFlags::MouseMovement | EventFlags::MouseButton,
      );
    }
  };

  let onclick = Closure::< dyn FnMut() >::new( closure );
  pathfind_button.set_onclick( Some( onclick.as_ref().unchecked_ref() ) );
  onclick.forget();


  ////// PAINTING DEMO //////

  let closure =
  {
    let input = input.clone();

    move ||
    {
      input.borrow_mut().clear_callbacks();

      context.clear( GL::COLOR_BUFFER_BIT );

      let canvas = canvas.clone();
      let context = context.clone();
      let hex_shader = hex_shader.clone();
      let hexagon_geometry = hexagon_geometry.clone();

      let mut painting_canvas = HexArray::< Offset< Odd >, Pointy, [ f32; 3 ] >::new
      (
        [ 23, 23 ].into(),
        [ 11, 11 ].into(),
        || [ 1.0, 1.0, 1.0 ]
      );

      let color_picker : HtmlInputElement = document
      .get_element_by_id( "color-picker" )
      .unwrap()
      .dyn_into()
      .unwrap();

      let closure = move | input : &input::InputState, _ |
      {
        let is_mouse_down = input.is_button_down( MouseButton::Main );

        if is_mouse_down
        {
          let rect = canvas.get_bounding_client_rect();
          let canvas_pos = F32x2::new( rect.left() as f32, rect.top() as f32 );
          let half_size : F32x2 = canvas_size / 2.0;
          let cursor_pos = F32x2::new( input.mouse_position()[ 0 ] as f32, input.mouse_position()[ 1 ] as f32 );
          let cursor_pos : Pixel = ( ( ( cursor_pos - canvas_pos ) - half_size ) / ( half_size * aspect_scale ) ).into();
          let selected_hex_coord : Coordinate::< Axial, Pointy > = cursor_pos.into();

          let color = color_picker.value();
          let r = u8::from_str_radix( &color[ 1..3 ], 16 ).unwrap() as f32 / 255.0;
          let g = u8::from_str_radix( &color[ 3..5 ], 16 ).unwrap() as f32 / 255.0;
          let b = u8::from_str_radix( &color[ 5..7 ], 16 ).unwrap() as f32 / 255.0;
          let color = [ r, g, b ];

          if painting_canvas[ selected_hex_coord ] == color { return; }

          painting_canvas[ selected_hex_coord ] = color;

          // draw painted hexagon
          let axial : Coordinate< Axial, _ > = selected_hex_coord.into();
          let hex_pos : Pixel = axial.into();
          let translation = mat2x2h::translate( [ hex_pos[ 0 ], -hex_pos[ 1 ] ] );
          let mvp = scale_m * translation * mat2x2h::rot( 30.0f32.to_radians() );
          hex_shader.uniform_matrix_upload( "u_mvp", mvp.raw_slice(), true );
          hex_shader.uniform_upload( "u_color", &[ r, g, b, 1.0 ] );
          hexagon_geometry.activate();
          context.draw_arrays( GL::TRIANGLES, 0, hexagon_geometry.nvertices );
        }
      };

      input.borrow_mut().add_callback
      (
        closure,
        EventFlags::MouseMovement | EventFlags::MouseButton,
      );
    }
  };

  let onclick = Closure::< dyn FnMut() >::new( closure );
  painting_button.set_onclick( Some( onclick.as_ref().unchecked_ref() ) );
  onclick.forget();

  Ok( () )
}
