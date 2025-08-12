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
use gl::{ JsCast as _, GL, I32x2, F32x2, Vector };
use std::{ cell::RefCell, rc::Rc, str::FromStr };
use tiles_tools::coordinates::pixel::Pixel;
use web_sys::HtmlCanvasElement;
use rustc_hash::FxHashMap;
use core_game::{ Coord };
use triaxial::TriAxial;
use helper::*;

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

  let config = include_str!( "../config.json" );
  let config = serde_json::from_str::< core_game::Config >( config ).unwrap();
  let player_colors = &config.player_colors
  .iter()
  .map( | [ r, g, b ] | [ *r as f32 / 255.0, *g as f32 / 255.0, *b as f32 / 255.0 ] )
  .collect::< Vec< _ > >();
  let mut textures = FxHashMap::default();
  load_textures_from_config( &document, &gl, &config, &mut textures );
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
    Some( canvas.dyn_into().unwrap() ),
    move | e |
    {
      let coord = gl::F64x2::new( e.client_x() as f64, e.client_y() as f64 ) * dpr;
      I32x2::from_array( [ coord.x() as i32, coord.y() as i32 ] )
    },
  );

  let water_color = [ 0.1, 0.2, 0.4 ];

  gl.clear_color( water_color[ 0 ], water_color[ 1 ], water_color[ 2 ], 1.0 );
  gl.viewport( 0, 0, width, height );
  gl.enable( GL::BLEND );
  gl.blend_func( GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA );

  let hexagon = create_hexagon_geometry( &gl )?;
  let outline = create_line_geometry( &gl )?;

  let hexagon_shader = hexagon_shader( &gl, player_colors.len() )?;
  let outline_shader = line_shader( &gl )?;
  let river_shader = river_shader( &gl )?;
  let sprite_shader = sprite_shader( &gl )?;
  let river_edge_shader = river_edge_shader( &gl )?;

  hexagon_shader.activate();
  hexagon_shader.uniform_upload( "u_player_colors", player_colors.as_slice() );
  river_shader.activate();
  river_shader.uniform_upload( "u_color", &water_color );
  river_edge_shader.activate();
  river_edge_shader.uniform_upload( "u_color", &water_color );

  let mut zoom = 1.0;
  let zoom_factor = 0.75;
  let inv_canvas_size = F32x2::new( 1.0 / width as f32, 1.0 / height as f32 );
  let aspect = width as f32 / height as f32;
  let aspect = F32x2::new( 1.0 / aspect, 1.0 );
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

    gl.clear( GL::COLOR_BUFFER_BIT );

    hexagon_shader.activate();
    hexagon_shader.uniform_upload( "u_scale", ( zoom * aspect ).as_slice() );
    hexagon_shader.uniform_upload( "u_camera_pos", camera_pos.as_slice() );

    outline_shader.activate();
    outline_shader.uniform_upload( "u_scale", ( zoom * aspect ).as_slice() );
    outline_shader.uniform_upload( "u_camera_pos", camera_pos.as_slice() );

    sprite_shader.activate();
    sprite_shader.uniform_upload( "u_scale", ( zoom * aspect ).as_slice() );
    sprite_shader.uniform_upload( "u_camera_pos", camera_pos.as_slice() );

    river_shader.activate();
    river_shader.uniform_upload( "u_scale", ( zoom * aspect ).as_slice() );
    river_shader.uniform_upload( "u_camera_pos", camera_pos.as_slice() );

    river_edge_shader.activate();
    river_edge_shader.uniform_upload( "u_scale", ( zoom * aspect ).as_slice() );
    river_edge_shader.uniform_upload( "u_camera_pos", camera_pos.as_slice() );

    for tile in map.borrow().tiles.values()
    {
      let mut position : Pixel = tile.coord.into();
      position.data[ 1 ] = -position.data[ 1 ];

      hexagon.bind( &gl );
      hexagon_shader.activate();
      gl.vertex_attrib2fv_with_f32_array( 1, &position.data );
      gl.vertex_attrib_i4i( 2, tile.owner_index.0 as i32, 0, 0, 0 );
      hexagon.draw( &gl );

      outline.bind( &gl );
      outline_shader.activate();
      gl.vertex_attrib2fv_with_f32_array( 1, &position.data );
      outline.draw( &gl );

      gl.bind_vertex_array( None );

      let Some( object_index ) = tile.object_index else { continue; };
      let prop = &config.object_props[ object_index.0 as usize ];
      let Some( sprite ) = &prop.sprite else { continue; };
      let texture = textures.get( &sprite.source ).unwrap();
      let size =
      [
        texture.size.borrow()[ 0 ] as f32 * 0.5,
        texture.size.borrow()[ 1 ] as f32 * 0.5
      ];
      gl.bind_texture( GL::TEXTURE_2D, texture.texture.as_ref() );
      sprite_shader.activate();
      gl.vertex_attrib2fv_with_f32_array( 0, &position.data );
      gl.vertex_attrib2fv_with_f32_array( 1, size.as_slice() );
      gl.vertex_attrib1f( 2, sprite.scale );
      gl.draw_arrays( GL::TRIANGLE_STRIP, 0, 4 );
    }

    gl.bind_vertex_array( None );

    let rivers = &map.borrow().rivers;
    let river_width = 0.1;
    river_shader.activate();
    for [ p1, p2 ] in rivers
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
      let rot = gl::math::d2::mat2x2h::rot( angle );
      let translate = gl::math::d2::mat2x2h::translate( center );
      let scale = gl::math::d2::mat2x2h::scale( [ length, river_width ] );
      let transform = translate * rot * scale;
      river_shader.uniform_matrix_upload( "u_transform", transform.raw_slice(), true );
      gl.draw_arrays( GL::TRIANGLE_STRIP, 0, 4 );
    }

    river_edge_shader.activate();
    for [ p1, p2 ] in rivers
    {
      let neighbors = p1.neighbors();
      let n1 = neighbors.iter().find( | n | **n != *p2 ).unwrap();
      let n2 = neighbors.iter().find( | n | **n != *p2 && **n != *n1 ).unwrap();

      let pair1 = [ *n1, *p1 ];
      let pair2 = [ *n2, *p1 ];

      let [ neighbor, _ ] = if ( rivers.contains( &pair1 ) && rivers.contains( &pair2 ) )
      || ( !rivers.contains( &pair1 ) && !rivers.contains( &pair2 ) )
      {
        continue;
      }
      else if rivers.contains( &pair1 )
      {
        pair1
      }
      else
      {
        pair2
      };

      let common_hexagon = p1.corners_axial()
      .into_iter()
      .find
      (
        | h | p2.corners_axial().contains( &h )
        && neighbor.corners_axial().contains( &h )
      ).unwrap();

      let origin : Pixel = common_hexagon.into();
      let point : Pixel = p1.to_point().into();
      let unit_point = point - origin;
      let angle = unit_point.y().atan2( unit_point.x() );
      let rot = gl::math::d2::mat2x2h::rot( -angle );
      let translate = gl::math::d2::mat2x2h::translate( &[ point.x(), -point.y() ] );
      river_edge_shader.uniform_upload( "u_width", &river_width );
      river_edge_shader.uniform_matrix_upload( "u_transform", ( translate * rot ).raw_slice(), true );
      gl.draw_arrays( GL::TRIANGLE_STRIP, 0, 4 );
    }

    true
  };
  gl::exec_loop::run( update );

  Ok( () )
}

fn load_textures_from_config
(
  document : &web_sys::Document,
  gl : &GL,
  config : &core_game::Config,
  textures : &mut rustc_hash::FxHashMap< String, Texture >
)
{
  textures.clear();
  for prop in &config.object_props
  {
    if let Some( sprite ) = &prop.sprite
    {
      if !textures.contains_key( &sprite.source )
      {
        let ( texture, size ) = load_texture( &gl, &document, &sprite.source );
        let texture = Texture { size, texture };
        textures.insert( sprite.source.clone(), texture );
      }
    }
  }
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
