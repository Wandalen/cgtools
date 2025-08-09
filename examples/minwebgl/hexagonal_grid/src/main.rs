//! Hexagonal grid pathfinding example using `tiles_tools` and `minwebgl`.
#![ allow( clippy::doc_markdown ) ]
#![ allow( clippy::wildcard_imports ) ] 
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::too_many_lines ) ]
#![ allow( clippy::default_trait_access ) ]
#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::needless_borrow ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::cast_possible_wrap ) ]
#![ allow( clippy::map_flatten ) ]

use minwebgl as min;
use browser_input::{ mouse, Input };
use tiles_tools::
{
  collection::Grid2D,
  coordinates::{ hexagonal::*, pixel::Pixel },
  geometry,
  layout::*
};
use min::
{
  math::{ F32x2, IntoVector, mat2x2h },
  Program,
  JsCast,
  canvas::HtmlCanvasElement,
  GL,
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
  .preserve_drawing_buffer( true );
  let context = min::context::retrieve_or_make_with( o )?;
  let canvas = context.canvas().unwrap().dyn_into::< HtmlCanvasElement >().unwrap();
  // used to scale canvas true size to css size
  let dpr = web_sys::window().unwrap().device_pixel_ratio() as f32;
  let canvas_size = ( canvas.width() as f32, canvas.height() as f32 ).into_vector() / dpr;

  let mut input = Input::new( Some( canvas.clone().dyn_into().unwrap() ), browser_input::CLIENT );

  // inclusize grid bounds
  let region =
  [
    Coordinate::< Offset< Odd >, Pointy >::new( 0, 0 ),
    Coordinate::< Offset< _ >, _ >::new( 8, 8 )
  ];
  let grid = RectangularGrid::new( region );
  // coordinates of a point in the center of grid
  let grid_center = grid.center();
  min::info!( "{:?}", grid_center );
  let aspect = canvas_size[ 1 ] / canvas_size[ 0 ];
  let scale = 0.1;
  let aspect_scale : F32x2 = [ aspect * scale, scale ].into();

  let grid_mesh = geometry::from_iter
  (
    grid.coordinates().map( | c | Into::< Coordinate< Axial, _ > >::into( c ) ),
    geometry::hexagon_triangles,
    // x is inverted because grid goes right and down, so to put it to center we need to offset it to left (neg x)
    // and up (pos y)
    mat2x2h::translate( [ -grid_center.x(), grid_center.y() ] )
    * mat2x2h::rot( 30.0f32.to_radians() )
    * mat2x2h::scale( [ 0.9, 0.9 ] )
  );

  let vert = include_str!( "../shaders/main.vert" );
  let frag = include_str!( "../shaders/main.frag" );
  let hex_shader = Rc::new( Program::new( context.clone(), vert, frag )? );
  hex_shader.activate();

  let grid_geometry = min::geometry::Positions::new
  (
    context.clone(),
    &grid_mesh,
    2,
  )?;
  let outline_geometry = min::geometry::Positions::new
  (
    context.clone(),
    &geometry::hexagon_lines(),
    2,
  )?;
  let hexagon_geometry = min::geometry::Positions::new
  (
    context.clone(),
    &geometry::hexagon_triangles(),
    2,
  )?;

  let document = web_sys::window().unwrap().document().unwrap();

  // used to swith between demos
  let demo_number = Rc::new( RefCell::new( 0 ) );

  let grid_button : HtmlButtonElement = document
  .get_element_by_id( "grid" )
  .unwrap()
  .dyn_into()
  .unwrap();
  let closure =
  {
    let demo_number = demo_number.clone();
    Closure::< dyn Fn() >::new
    (
      move ||
      {
        *demo_number.borrow_mut() = 0;
      }
    )
  };
  grid_button.set_onclick( Some( closure.as_ref().unchecked_ref() ) );
  closure.forget();

  let pathfind_button : HtmlButtonElement = document
  .get_element_by_id( "pathfinding" )
  .unwrap()
  .dyn_into()
  .unwrap();
  let closure =
  {
    let demo_number = demo_number.clone();
    Closure::< dyn Fn() >::new
    (
      move ||
      {
        *demo_number.borrow_mut() = 1;
      }
    )
  };
  pathfind_button.set_onclick( Some( closure.as_ref().unchecked_ref() ) );
  closure.forget();

  let painting_button : HtmlButtonElement = document
  .get_element_by_id( "painting" )
  .unwrap()
  .dyn_into()
  .unwrap();
  let closure =
  {
    let demo_number = demo_number.clone();
    let context = context.clone();
    Closure::< dyn Fn() >::new
    (
      move ||
      {
        *demo_number.borrow_mut() = 2;
        context.clear( GL::COLOR_BUFFER_BIT );
      }
    )
  };
  painting_button.set_onclick( Some( closure.as_ref().unchecked_ref() ) );
  closure.forget();

  let color_picker : HtmlInputElement = document
  .get_element_by_id( "color-picker" )
  .unwrap()
  .dyn_into()
  .unwrap();

  // for pathfind demo
  let mut start = Coordinate::< Axial, Pointy >::new( 2, 4 );
  let mut obstacles = HashMap::< Coordinate< Axial, Pointy >, bool >::from_iter
  (
    grid.coordinates().map( | c | ( c.into(), true ) )
  );

  // array to store painted hexagons
  let mut painting_canvas = Grid2D::< Offset< Odd >, Pointy, [ f32; 3 ] >::with_size_and_fn
  (
    [ -11, -11 ].into(),
    [ 12, 12 ].into(),
    || [ 1.0, 1.0, 1.0 ]
  );

  let draw = move | _ |
  {
    input.update_state();

    let rect = canvas.get_bounding_client_rect();
    let canvas_pos = F32x2::new( rect.left() as f32, rect.top() as f32 );
    let half_size = canvas_size / 2.0;
    let cursor_pos = F32x2::new( input.pointer_position()[ 0 ] as f32, input.pointer_position()[ 1 ] as f32 );
    // normalize coodinates to NDC [ -1 : 1 ], then apply inverse ascpect scale
    // this transforms cursor position to world space
    // then offset it by center of the grid, so that if cursor is in the center of the canvas, it will be in the center of the grid
    let cursor_pos : Pixel =
    (
      ( ( cursor_pos - canvas_pos ) - half_size ) / ( half_size * aspect_scale ) + F32x2::from_array( grid_center.data )
    ).into();
    // hexagon which cursor points to
    let selected_hex_coord : Coordinate::< Axial, Pointy > = cursor_pos.into();

    match *demo_number.borrow()
    {
      0 =>
      {
        grid_demo
        (
          &context,
          grid_center,
          aspect_scale,
          &hex_shader,
          &grid_geometry,
          &outline_geometry,
          selected_hex_coord
        );
      }
      1 =>
      {
        pathfind_demo
        (
          &context,
          &input,
          grid_center,
          aspect_scale,
          &hex_shader,
          &grid_geometry,
          &hexagon_geometry,
          &mut start,
          &mut obstacles,
          selected_hex_coord
        );
      }
      _ =>
      {
        painting_demo
        (
          &context,
          &canvas,
          canvas_size,
          &input,
          aspect_scale,
          &hex_shader,
          &hexagon_geometry,
          &mut painting_canvas,
          &color_picker
        );
      }
    }

    input.clear_events();

    true
  };

  min::exec_loop::run( draw );

  Ok( () )
}

// function responsible for painting on grid
fn painting_demo
(
  context : &GL,
  canvas : &HtmlCanvasElement,
  canvas_size :F32x2,
  input : &Input,
  scale : F32x2,
  hex_shader : &Program,
  hexagon_geometry : &min::geometry::Positions,
  painting_canvas : &mut Grid2D< Offset< Odd >, Pointy, [ f32; 3 ] >,
  color_picker : &HtmlInputElement
)
{
  let is_mouse_down = input.is_button_down( mouse::MouseButton::Main );
  // not painting anything if the mouse is not pressed
  if !is_mouse_down
  {
    return;
  }

  // calculate pixel coordinates
  let rect = canvas.get_bounding_client_rect();
  let canvas_pos = F32x2::new( rect.left() as f32, rect.top() as f32 );
  let half_size : F32x2 = canvas_size / 2.0;
  let cursor_pos = F32x2::new
  (
    input.pointer_position()[ 0 ] as f32,
    input.pointer_position()[ 1 ] as f32
  );
  let pos : Pixel =
  (
    ( ( cursor_pos - canvas_pos ) - half_size ) / ( half_size * scale )
  ).into();
  // calculate hex coordinates
  let selected_hex_coord : Coordinate::< Axial, Pointy > = pos.into();

  // get color
  let color = color_picker.value();
  let r = u8::from_str_radix( &color[ 1..3 ], 16 ).unwrap() as f32 / 255.0;
  let g = u8::from_str_radix( &color[ 3..5 ], 16 ).unwrap() as f32 / 255.0;
  let b = u8::from_str_radix( &color[ 5..7 ], 16 ).unwrap() as f32 / 255.0;
  let color = [ r, g, b ];

  painting_canvas[ selected_hex_coord ] = color;

  // draw painted hexagon
  let axial : Coordinate< Axial, _ > = selected_hex_coord.into();
  let pos : Pixel = axial.into();
  let angle = 30.0f32.to_radians();

  // inverse y so it points up
  context.vertex_attrib2f( 1, pos.x(), -pos.y() );
  hex_shader.uniform_upload( "u_zoom", scale.as_slice() );
  hex_shader.uniform_upload( "u_rotation", [ angle.cos(), angle.sin() ].as_slice() );
  hex_shader.uniform_upload( "u_color", &[ r, g, b, 1.0 ] );

  hexagon_geometry.activate();
  // disable attribute used for instancing
  context.vertex_attrib2f( 1, pos.x(), -pos.y() );
  context.disable_vertex_attrib_array( 1 );

  context.draw_arrays( GL::TRIANGLES, 0, hexagon_geometry.nvertices );
}

// function responsible for demonstrating pathfind in grid
fn pathfind_demo
(
  context : &GL,
  input : &Input,
  mut grid_center : Pixel,
  scale : min::F32x2,
  hex_shader : &Program,
  grid_geometry : &min::geometry::Positions,
  hexagon_geometry : &min::geometry::Positions,
  start : &mut Coordinate< Axial, Pointy >,
  obstacles : &mut HashMap< Coordinate< Axial, Pointy >, bool >,
  selected_hex_coord : Coordinate< Axial, Pointy >
)
{
  // update obstacles and start position
  for browser_input::Event { event_type, .. } in input.event_queue().as_slice()
  {
    if let browser_input::EventType::MouseButton( button, browser_input::Action::Press ) = event_type
    {
      if *button == mouse::MouseButton::Main
      && obstacles.contains_key( &selected_hex_coord )
      && selected_hex_coord != *start
      {
        obstacles.entry( selected_hex_coord ).and_modify( | v | *v = !*v );
      }
      if *button == mouse::MouseButton::Auxiliary
      && obstacles.get( &selected_hex_coord ).copied().unwrap_or_default()
      {
        *start = selected_hex_coord;
      }
    }
  }

  context.clear( GL::COLOR_BUFFER_BIT );

  // draw background grid
  context.vertex_attrib2f( 1, 0.0, 0.0 );
  hex_shader.uniform_upload( "u_zoom", scale.as_slice() );
  // no need for rotation here
  hex_shader.uniform_upload( "u_rotation", [ 1.0, 0.0 ].as_slice() );
  hex_shader.uniform_upload( "u_color", &[ 0.7, 0.7, 0.7, 1.0 ] );
  grid_geometry.activate();
  context.draw_arrays( GL::TRIANGLES, 0, grid_geometry.nvertices );

  // rotation to make hexagons pointy
  let angle : f32 = 30.0f32.to_radians();
  // invert y so it points upward
  grid_center[ 1 ] = -grid_center[ 1 ];

  // draw obstacles
  let offsets = obstacles
  .iter()
  .filter( | ( _, v ) | !**v )
  .map( | ( coord, _ ) |
  {
    let mut pos : Pixel = ( *coord ).into();
    // y points down
    pos[ 1 ] = -pos[ 1 ];
    ( pos - grid_center ).data
  })
  .flatten()
  .collect::< Vec< _ > >();
  let count = ( offsets.len() / 2 ) as i32;

  hexagon_geometry.activate();
  let offsets_buffer = min::buffer::create( &context ).unwrap();
  min::buffer::upload( &context, &offsets_buffer, offsets.as_slice(), GL::DYNAMIC_DRAW );
  min::BufferDescriptor::new::< [ f32; 2 ] >()
  .offset( 0 )
  .stride( 0 )
  .divisor( 1 )
  .attribute_pointer( &context, 1, &offsets_buffer ).unwrap();

  hex_shader.uniform_upload( "u_zoom", scale.as_slice() );
  hex_shader.uniform_upload( "u_rotation", [ angle.cos(), angle.sin() ].as_slice() );
  hex_shader.uniform_upload( "u_color", &[ 0.1, 0.1, 0.1, 1.0 ] );
  context.draw_arrays_instanced( GL::TRIANGLES, 0, hexagon_geometry.nvertices, count );

  let goal = selected_hex_coord;

  let path = tiles_tools::pathfind::astar
  (
    start,
    &goal,
    | coord | obstacles.get( &coord ).copied().unwrap_or_default(),
    | _ | 1
  );

  let Some( ( path, _ ) ) = path else
  {
    return;
  };

  let offsets = path.iter().map( | coord |
  {
    let mut pos : Pixel = ( *coord ).into();
    pos[ 1 ] = -pos[ 1 ];
    ( pos - grid_center ).data
  })
  .flatten()
  .collect::< Vec< _ > >();
  let count = ( offsets.len() / 2 ) as i32;
  min::buffer::upload( &context, &offsets_buffer, offsets.as_slice(), GL::DYNAMIC_DRAW );

  hex_shader.uniform_upload( "u_mvp", scale.as_slice() );
  hex_shader.uniform_upload( "u_rotation", [ angle.cos(), angle.sin() ].as_slice() );
  hex_shader.uniform_upload( "u_color", &[ 0.1, 0.6, 0.1, 1.0 ] );
  context.draw_arrays_instanced( GL::TRIANGLES, 0, hexagon_geometry.nvertices, count );
}

// function responsible for demonstrating grid
fn grid_demo
(
  context : &GL,
  mut grid_center : Pixel,
  scale : min::F32x2,
  hex_shader : &Program,
  grid_geometry : &min::geometry::Positions,
  outline_geometry : &min::geometry::Positions,
  selected_hex_coord : Coordinate< Axial, Pointy >
)
{
  context.clear( GL::COLOR_BUFFER_BIT );

  // draw grid
  context.vertex_attrib2f( 1, 0.0, 0.0 );
  hex_shader.uniform_upload( "u_zoom", scale.as_slice() );
  hex_shader.uniform_upload( "u_color", &[ 0.7, 0.7, 0.7, 1.0 ] );
  hex_shader.uniform_upload( "u_rotation", [ 1.0, 0.0 ].as_slice() );
  grid_geometry.activate();
  context.draw_arrays( GL::TRIANGLES, 0, grid_geometry.nvertices );

  // make y negative because it actualy points down
  grid_center[ 1 ] = -grid_center[ 1 ];

  let mut selected_hex_pos : Pixel = selected_hex_coord.into();
  // same thing here
  selected_hex_pos[ 1 ] = -selected_hex_pos[ 1 ];

  // draw outline
  let translation = selected_hex_pos - grid_center;
  // rotate hexagon by 30 deg so it is pointy
  let angle = 30.0f32.to_radians();
  context.vertex_attrib2f( 1, translation.x(), translation.y() );
  hex_shader.uniform_upload( "u_zoom", scale.as_slice() );
  hex_shader.uniform_upload( "u_rotation", [ angle.cos(), angle.sin() ].as_slice() );
  hex_shader.uniform_upload( "u_color", &[ 0.1, 0.1, 0.1, 1.0 ] );
  outline_geometry.activate();
  context.draw_arrays( GL::LINES, 0, outline_geometry.nvertices );
}
