//! This crate provides a simple hexagonal map editor using WebGL and browser input handling.
//! It allows users to edit tiles, rivers, and player colors on a hexagonal grid.
//! The map can be saved and loaded in JSON format.

#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::default_trait_access ) ]
#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::assign_op_pattern ) ]
#![ allow( clippy::semicolon_if_nothing_returned ) ]
#![ allow( clippy::too_many_lines ) ]
#![ allow( clippy::wildcard_imports ) ]
#![ allow( clippy::needless_borrow ) ]
#![ allow( clippy::cast_possible_wrap ) ]
#![ allow( clippy::redundant_field_names ) ]
#![ allow( clippy::useless_format ) ]
#![ allow( clippy::let_unit_value ) ]
#![ allow( clippy::needless_return ) ]
#![ allow( clippy::cast_sign_loss ) ]
#![ allow( clippy::similar_names ) ]
#![ allow( clippy::needless_continue ) ]
#![ allow( clippy::else_if_without_else ) ]
#![ allow( clippy::unreadable_literal ) ]
#![ allow( clippy::explicit_iter_loop ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::collapsible_if ) ]
#![ allow( clippy::unused_async ) ]
#![ allow( clippy::needless_borrows_for_generic_args ) ]
#![ allow( clippy::wrong_self_convention ) ]
#![ allow( clippy::neg_multiply ) ]
#![ allow( clippy::cast_lossless ) ]
#![ allow( clippy::needless_pass_by_value ) ]
#![ allow( clippy::excessive_precision ) ]

mod helper;
mod triaxial;
mod core_game;

use minwebgl as gl;
use browser_input::{ keyboard::KeyboardKey, mouse::MouseButton, Action, Event, EventType };
use gl::{ JsCast as _, I32x2, F32x2, Vector };
use tilemap_renderer::{ adapters::WebGLTileRenderer, commands };
use commands::Transform2D;
use std::{ cell, rc::Rc, str::FromStr };
use cell::{ Cell, RefCell };
use tiles_tools::coordinates::pixel::Pixel;
use web_sys::HtmlCanvasElement;
use rustc_hash::FxHashMap;
use core_game::{ Coord };
use triaxial::TriAxial;
use helper::*;

type TexSize =  Rc< Cell< [ u32; 2 ] > >;

fn main()
{
  gl::browser::setup( Default::default() );
  gl::spawn_local( async move { run().await.unwrap() } );
}

async fn run() -> Result< (), gl::WebglError >
{
  let window = web_sys::window().unwrap();
  let document = window.document().unwrap();
  let fwidth = window.inner_width().unwrap().as_f64().unwrap();
  let fheight = window.inner_height().unwrap().as_f64().unwrap();
  let dpr = window.device_pixel_ratio();
  let gl = gl::context::retrieve_or_make().unwrap();

  let canvas = gl.canvas().unwrap().dyn_into::< HtmlCanvasElement >().unwrap();
  let width = ( fwidth * dpr ) as i32;
  let height = ( fheight * dpr ) as i32;
  canvas.set_width( width as u32 );
  canvas.set_height( height as u32 );
  browser_input::prevent_rightclick( canvas.clone().dyn_into().unwrap() );

  let default = tilemap_renderer::ports::RenderContext::new
  (
    0,
    0,
    [ 0.0; 4 ],
    true,
    tilemap_renderer::commands::Point2D { x : 0.0, y : 0.0  },
    0.0
  );
  let mut renderer = WebGLTileRenderer::new( &gl, default ).unwrap();

  let hexagon_id = 0;
  let outline_id = 1;
  let rectangle_id = 2;
  let rectangle : &[ f32 ] = &
  [
    -1.0, -1.0,
     1.0,  1.0,
    -1.0,  1.0,
    -1.0, -1.0,
     1.0, -1.0,
     1.0,  1.0,
  ];
  let hexagon_mesh = tiles_tools::geometry::hexagon_triangles();
  let outline_mesh = tiles_tools::geometry::hexagon_lines();
  renderer.geometry2d_load( &hexagon_mesh, hexagon_id ).unwrap();
  renderer.geometry2d_load( &outline_mesh, outline_id ).unwrap();
  renderer.geometry2d_load( rectangle, rectangle_id ).unwrap();

  let config = include_str!( "../config.json" );
  let config = serde_json::from_str::< core_game::Config >( config ).unwrap();

  let mut textures = FxHashMap::< String, ( u32, TexSize ) >::default();
  for object in &config.object_props
  {
    let Some( sprite ) = &object.sprite else { continue; };
    if textures.contains_key( &sprite.source ) { continue; }
    let id = textures.len() as u32;
    let size = renderer
    .texture_load_from_src( &document, &sprite.source, id )
    .expect( "Failed to load textures" );
    textures.insert( sprite.source.clone(), ( id, size ) );
  }

  let map = Rc::new( RefCell::new( core_game::Map::default() ) );
  let loaded_map : Rc< RefCell < Option< String > > > = Default::default();

  // Setup select elements
  let mode_select_variants = [ EditMode::EditTiles, EditMode::EditRivers ].map( | v | v.as_ref().to_string() );
  let mode_select = setup_select( &document, "edit-mode", mode_select_variants.iter() );
  let tile_select = setup_select( &document, "tile", config.object_props.iter().map( | p | &p.name ) );
  let player_list = config.player_colors.iter().enumerate().map( | ( i, _ ) |  i.to_string() ).collect::< Vec< _ > >();
  let player_select = setup_select( &document, "player", player_list.iter() );

  setup_download_button( &document, map.clone() );
  setup_drop_zone( &document, loaded_map.clone() );

  let mut input = browser_input::Input::new
  (
    Some( canvas.clone().dyn_into().unwrap() ),
    move | e |
    {
      let coord = gl::F64x2::new( e.client_x() as f64, e.client_y() as f64 ) * dpr;
      I32x2::from_array( [ coord.x() as i32, coord.y() as i32 ] )
    },
  );

  let water_color = [ 0.1, 0.2, 0.4 ];

  let mut zoom = 0.1;
  let zoom_factor = 0.75;
  let inv_canvas_size = F32x2::new( 1.0 / width as f32, 1.0 / height as f32 );
  let aspect = if width > height
  {
    F32x2::from_array( [ 1.0, ( width as f32 / height as f32 ) ] )
  }
  else
  {
    F32x2::from_array( [ ( height as f32 / width as f32 ), 1.0 ] )
  };
  let mut camera_pos = F32x2::default();

  let mut last_pointer_pos : Option< I32x2 > = None;
  let mut river_point1_add = None;
  let mut river_point1_remove = None;

  let update = move | _ |
  {
    input.update_state();

    let mut loaded_map = loaded_map.borrow_mut();
    if let Some( map_json ) = loaded_map.as_ref()
    {
      match serde_json::from_str( &map_json )
      {
        Ok( m ) => *map.borrow_mut() = m,
        Err( e ) => gl::warn!( "{e}" ),
      }
      *loaded_map = None;
    }

    for Event { event_type, .. } in input.event_queue().iter()
    {
      if let EventType::Wheel( Vector( [ _, delta, _ ] ) ) = event_type
      {
        if delta.is_sign_negative()
        {
          zoom /= zoom_factor;
        }
        else
        {
          zoom *= zoom_factor;
        }
      }
    }

    let pointer_pos = input.pointer_position();
    let pointer_pos = screen_to_world( pointer_pos, inv_canvas_size, aspect, zoom );
    let mut pixel = pointer_pos - camera_pos;
    pixel[ 1 ] = -pixel[ 1 ];
    let pixel : Pixel = pixel.into();
    let hexagon_coord : Coord = pixel.into();
    let tri_point = TriAxial::from_point( pixel.x(), pixel.y() );

    let edit_mode = EditMode::from_str( &mode_select.value() ).unwrap();

    if edit_mode == EditMode::EditRivers
    {
      for Event { event_type, .. } in input.event_queue().iter()
      {
        if let EventType::MouseButton( MouseButton::Main, Action::Press ) = event_type
        {
          if river_point1_add.is_none()
          {
            river_point1_add = Some( tri_point );
          }
          else
          {
            let river_point1 = river_point1_add.take().unwrap();
            let river_point2 = tri_point;

            if river_point1.neighbors().contains( &river_point2 )
            {
              map.borrow_mut().rivers.insert( [ river_point1, river_point2 ] );
            }
          }
        }
        if let EventType::MouseButton( MouseButton::Secondary, Action::Press ) = event_type
        {
          if river_point1_remove.is_none()
          {
            river_point1_remove = Some( tri_point );
          }
          else
          {
            let river_point1 = river_point1_remove.take().unwrap();
            let river_point2 = tri_point;
            if river_point1.neighbors().contains( &river_point2 )
            {
              map.borrow_mut().rivers.remove( &[ river_point1, river_point2 ] );
            }
          }
        }
      }
    }

    if input.is_key_down( KeyboardKey::Space ) && input.is_button_down( MouseButton::Main )
    {
      if let Some( last_pointer_pos ) = last_pointer_pos
      {
        let last_pointer_pos = screen_to_world
        (
          last_pointer_pos,
          inv_canvas_size,
          aspect,
          zoom
        );
        let movement = pointer_pos - last_pointer_pos;
        camera_pos += movement;
      }
    }
    else if edit_mode == EditMode::EditTiles
    {
      let main_button = input.is_button_down( MouseButton::Main );
      let secondary_button = input.is_button_down( MouseButton::Secondary );
      let selected_value = tile_select.value();
      let object_index = core_game::ObjectIndex
      (
        config.object_props.iter().position( | p | p.name == selected_value ).unwrap() as u32
      );
      let owner_index = core_game::PlayerIndex( player_select.value().parse().unwrap() );

      if main_button
      {
        let tile = core_game::Tile
        {
          object_index : Some( object_index ),
          terrain_index: Default::default(),
          owner_index,
          coord : hexagon_coord,
        };
        map.borrow_mut().tiles.insert( hexagon_coord, tile );
      }
      else if secondary_button
      {
        map.borrow_mut().tiles.remove( &hexagon_coord );
      }
    }

    last_pointer_pos = Some( input.pointer_position() );
    input.clear_events();

    let width = canvas.width();
    let height = canvas.height();
    let ctx = tilemap_renderer::ports::RenderContext::new
    (
      width,
      height,
      [ 0.1, 0.2, 0.3, 1.0 ],
      true,
      tilemap_renderer::commands::Point2D { x : camera_pos[ 0 ], y : camera_pos[ 1 ]  },
      zoom
    );
    renderer.context_set( ctx );

    let mut commands = vec![];

    for hex in map.borrow().tiles.values()
    {
      let mut position : Pixel = hex.coord.into();
      position.data[ 1 ] = -position.data[ 1 ];
      let tr = Transform2D::new( position.data, 0.0, [ 1.0, 1.0 ] );
      let [ r, g, b ] = config.player_colors[ hex.owner_index.0 as usize ];
      let color = [ f32::from( r ) / 255.0, f32::from( g ) / 255.0, f32::from( b ) / 255.0 ];
      let command = commands::Geometry2DCommand
      {
        id : hexagon_id,
        transform : tr,
        color,
        mode : commands::GeometryMode::Triangles,
      };
      commands.push( commands::RenderCommand::Geometry2DCommand( command ) );

      let color = [ 0.0_f32; 3 ];
      let command = commands::Geometry2DCommand
      {
        id : outline_id,
        transform : tr,
        color,
        mode : commands::GeometryMode::Lines,
      };
      commands.push( commands::RenderCommand::Geometry2DCommand( command ) );

      let Some( object_index ) = hex.object_index else { continue; };

      let object = &config.object_props[ object_index.0 as usize ];

      let Some( sprite ) = &object.sprite else { continue; };

      let ( id, size ) = &textures[ &sprite.source ];
      let scale = sprite.scale;
      let scale =
      [
        scale * 0.5 * size.get()[ 0 ] as f32,
        scale * 0.5 * size.get()[ 1 ] as f32
      ];

      let mut position : Pixel = hex.coord.into();
      position.data[ 1 ] = -position.data[ 1 ];
      let tr = Transform2D::new( position.data, 0.0, scale );
      let c = commands::SpriteCommand
      {
        id : *id,
        transform : tr,
      };
      commands.push( commands::RenderCommand::SpriteCommand( c ) );
    }

    let river_width = 0.1;
    for [ p1, p2 ] in &map.borrow().rivers
    {
      let mut p1 : F32x2 = p1.to_point().into();
      p1[ 1 ] = -p1[ 1 ];
      let mut p2 : F32x2 = p2.to_point().into();
      p2[ 1 ] = -p2[ 1 ];
      let center = ( p1 + p2 ) / 2.0;
      let length = p1.distance( &p2 ) / 2.0;
      let dx = p2.x() - p1.x();
      let dy = p2.y() - p1.y();
      let angle = dy.atan2( dx );

      let tr = Transform2D::new( center, angle, [ length, river_width ].into() );
      let color = water_color;
      let command = commands::Geometry2DCommand
      {
        id : rectangle_id,
        transform : tr,
        color,
        mode : commands::GeometryMode::Triangles,
      };
      commands.push( commands::RenderCommand::Geometry2DCommand( command ) );
    }

    renderer.commands_execute( &commands );

    true
  };
  gl::exec_loop::run( update );

  Ok( () )
}

fn screen_to_world
(
  screen : I32x2,
  inv_canvas_size : F32x2,
  aspect : F32x2,
  zoom : f32
) -> F32x2
{
  let Vector ( [ x, y ] ) = screen;
  let screenf32 = F32x2::new( x as f32, y as f32 );
  let mut world = ( screenf32 * inv_canvas_size - 0.5 ) * 2.0 / ( zoom * aspect );
  world[ 1 ] = -world[ 1 ];
  world
}
