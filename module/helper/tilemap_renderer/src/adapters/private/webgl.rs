//! WebGL backend adapter implementation.
//!
//! This adapter provides hardware-accelerated rendering using WebGL via WebAssembly,
//! designed to integrate with the minwebgl crate for optimal GPU performance.

#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::field_reassign_with_default ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::cast_sign_loss ) ]
#![ allow( clippy::similar_names ) ]
#![ allow( clippy::missing_docs_in_private_items ) ]
#![ allow( clippy::unreadable_literal ) ]
#![ allow( clippy::missing_inline_in_public_items ) ]
#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::needless_return ) ]
#![ allow( clippy::needless_borrow ) ]
#![ allow( clippy::unnecessary_wraps ) ]
#![ allow( clippy::no_effect_underscore_binding ) ]

use minwebgl as gl;
use gl::JsCast as _;
use std::{ cell, rc };
use web_sys::wasm_bindgen::prelude::Closure;
use crate::{ commands, ports };
// use crate::scene::Scene;
// use crate::commands::{ RenderCommand, LineCommand, CurveCommand, TextCommand, TextAnchor, TilemapCommand, ParticleEmitterCommand };
// use ports::{ PrimitiveRenderer, RenderContext, RenderError, Renderer, RendererCapabilities };

/// The WebGL renderer tile rendering logic.
///
/// This struct manages GPU resources such as textures, geometry, and shader programs.
/// It processes a queue of `RenderCommand`s to draw to the canvas.
#[ derive( Debug ) ]
pub struct WebGLTileRenderer
{
  gl : gl::GL,
  geometry2d : rustc_hash::FxHashMap< u32, ( web_sys::WebGlVertexArrayObject, u32 ) >,
  textures : rustc_hash::FxHashMap< u32, web_sys::WebGlTexture >,
  geom2d_program : gl::Program,
  sprite_program : gl::Program,
  context : ports::RenderContext,
}

impl WebGLTileRenderer
{
  /// Creates a new renderer instance.
  ///
  /// This function initializes the WebGL shader programs and sets up the initial state.
  #[ inline ]
  pub fn new( gl : &gl::GL, render_context : ports::RenderContext ) -> Result< Self, gl::WebglError >
  {
    let v_main = include_str!( "../../../shaders/main.vert" );
    let f_main = include_str!( "../../../shaders/main.frag" );
    let geom2d_program = gl::Program::new( gl.clone(), v_main, f_main )?;

    let v_sprite = include_str!( "../../../shaders/sprite.vert" );
    let f_sprite = include_str!( "../../../shaders/sprite.frag" );
    let sprite_program = gl::Program::new( gl.clone(), v_sprite, f_sprite )?;

    Ok
    (
      Self
      {
        gl : gl.clone(),
        geometry2d : rustc_hash::FxHashMap::default(),
        textures : rustc_hash::FxHashMap::default(),
        geom2d_program,
        sprite_program,
        context : render_context,
      }
    )
  }

  /// Creates a vertex buffer, loads `data` into it, and stores it internally by the provided `id`.
  ///
  /// Expects a slice of `f32` representing 2D points. The `id` can be used to render the geometry later.
  /// If geometry with the same `id` already exists, it is replaced.
  #[ inline ]
  pub fn geometry2d_load( &mut self, data : &[ f32 ], id : u32 ) -> Result< (), gl::WebglError >
  {
    let gl = &self.gl;
    let vao = gl::vao::create( gl )?;
    gl.bind_vertex_array( Some( &vao ) );
    let buffer = gl::buffer::create( gl )?;
    gl::buffer::upload( gl, &buffer, data, gl::GL::STATIC_DRAW );
    gl::BufferDescriptor::new::< [ f32; 2 ] >()
    .attribute_pointer( gl, 0, &buffer )?;
    gl.bind_vertex_array( None );
    let vertex_count =  data.len() as u32 / 2;

    _ = self.geometry2d.insert( id, ( vao, vertex_count ) );

    Ok( () )
  }

  /// Asynchronously loads an image from a URL, creates a WebGL texture, and stores it internally by `id`.
  /// If texture with the same `id` already exists, it is replaced.
  ///
  /// # Returns
  ///
  /// A `Result` containing a shared reference to the texture's dimensions, which will be populated
  /// once the image loads.
  #[ inline ]
  pub fn texture_load_from_src( &mut self, document : &web_sys::Document, src : &str, id : u32 )
  -> Result< rc::Rc< cell::Cell< [ u32; 2 ] > >, gl::WebglError >
  {
    let img = document.create_element( "img" )
    .unwrap()
    .dyn_into::< web_sys::HtmlImageElement >()
    .unwrap();
    img.style().set_property( "display", "none" ).unwrap();

    let gl = &self.gl;

    let texture = gl.create_texture()
    .ok_or( gl::WebglError::FailedToAllocateResource( "Failed to allocate texture" ) )?;

    let size = rc::Rc::new( cell::Cell::new( [ 0; 2 ] ) );

    let on_load : Closure< dyn Fn() > = Closure::new
    ({
      let gl = gl.clone();
      let img = img.clone();
      let texture = texture.clone();
      let size = size.clone();
      move ||
      {
        let width = img.natural_width();
        let height = img.natural_height();
        size.set( [ width, height ] );
        gl::texture::d2::upload( &gl, Some( &texture ), &img );
        gl::texture::d2::filter_linear( &gl );
        gl::texture::d2::wrap_clamp( &gl );
        img.remove();
      }
    });

    img.set_onload( Some( on_load.as_ref().unchecked_ref() ) );
    img.set_src( src );
    on_load.forget();

    _ = self.textures.insert( id, texture );

    Ok( size )
  }

  /// Sets the `RenderContext`.
  #[ inline ]
  pub fn context_set( &mut self, render_context : ports::RenderContext )
  {
    self.context = render_context;
  }

  /// Executes a list of render commands for the current frame.
  ///
  /// Currently supports only `RenderCommand::Geometry2DCommand`, `RenderCommand::SpriteCommand`
  /// commands. In case of facing an unsupported command in the command buffer just ignores it.
  #[ inline ]
  pub fn commands_execute( &self, commands : &[ commands::RenderCommand ] )
  {
    let ctx = &self.context;
    let width = ctx.width;
    let height = ctx.height;
    let scale = ctx.viewport_scale;
    let camera_pos = [ ctx.viewport_offset.x, ctx.viewport_offset.y ];
    let aspect_scale = if width > height
    {
      [ scale, ( width as f32 / height as f32 ) * scale ]
    }
    else
    {
      [ ( height as f32 / width as f32 ) * scale, scale ]
    };

    let gl = &self.gl;
    let [ r, g, b, a ] = ctx.background_color;
    gl.clear_color( r, g, b, a );
    gl.viewport( 0, 0, ctx.width as i32, ctx.height as i32 );
    gl.enable( gl::GL::BLEND );
    gl.blend_func( gl::GL::ONE, gl::GL::ONE_MINUS_SRC_ALPHA );

    if ctx.clear_background
    {
      gl.clear( gl::GL::COLOR_BUFFER_BIT );
    }

    for command in commands
    {
      match command
      {
        commands::RenderCommand::Geometry2DCommand( command ) =>
        {
          self.geometry2d_draw( &command, camera_pos, aspect_scale )
        }
        commands::RenderCommand::SpriteCommand( command ) =>
        {
          self.sprite_draw( &command, camera_pos, aspect_scale );
        },
        commands::RenderCommand::Line( _ ) |
        commands::RenderCommand::Curve( _ ) |
        commands::RenderCommand::Text( _ ) |
        commands::RenderCommand::Tilemap( _ ) |
        commands::RenderCommand::ParticleEmitter( _ ) => {}
      }
    }
  }

  fn geometry2d_draw
  (
    &self,
    command : &commands::Geometry2DCommand,
    camera_pos : [ f32; 2 ],
    aspect_scale : [ f32; 2 ]
  )
  {
    let [ pos_x, pos_y ] = command.transform.position;
    let [ cam_pos_x, cam_pos_y ] = camera_pos;
    let translation = [ pos_x + cam_pos_x, pos_y + cam_pos_y ];
    let rotation_cos_sin = [ command.transform.rotation.cos(), command.transform.rotation.sin() ];

    let gl = &self.gl;
    let ( vao, vertex_count ) = &self.geometry2d[ &command.id ];
    gl.bind_vertex_array( Some( &vao ) );

    self.geom2d_program.activate();
    gl.vertex_attrib2fv_with_f32_array( 1, &translation );
    gl.vertex_attrib2fv_with_f32_array( 2, &rotation_cos_sin );
    gl.vertex_attrib2fv_with_f32_array( 3, &command.transform.scale );
    self.geom2d_program.uniform_upload( "u_aspect_scale", &aspect_scale );
    self.geom2d_program.uniform_upload( "u_color", &command.color );

    let mode = match command.mode
    {
      commands::GeometryMode::Triangles => gl::GL::TRIANGLES,
      commands::GeometryMode::Lines => gl::GL::LINES,
    };

    gl.draw_arrays( mode, 0, *vertex_count as i32 );
  }

  fn sprite_draw
  (
    &self,
    command : &commands::SpriteCommand,
    camera_pos : [ f32; 2 ],
    aspect_scale : [ f32; 2 ]
  )
  {
    let [ pos_x, pos_y ] = command.transform.position;
    let [ cam_pos_x, cam_pos_y ] = camera_pos;
    let translation = [ pos_x + cam_pos_x, pos_y + cam_pos_y ];
    let rotation_cos_sin = [ command.transform.rotation.cos(), command.transform.rotation.sin() ];

    let gl = &self.gl;
    let texture = &self.textures[ &command.id ];
    gl.active_texture( gl::GL::TEXTURE0 );
    gl.bind_texture( gl::GL::TEXTURE_2D, Some( texture ) );
    gl.bind_vertex_array( None );

    self.sprite_program.activate();
    gl.vertex_attrib2fv_with_f32_array( 1, &translation );
    gl.vertex_attrib2fv_with_f32_array( 2, &rotation_cos_sin );
    gl.vertex_attrib2fv_with_f32_array( 3, &command.transform.scale );

    self.sprite_program.uniform_upload( "u_aspect_scale", &aspect_scale );
    self.sprite_program.uniform_upload( "u_color", &0 );

    gl.draw_arrays( gl::GL::TRIANGLE_STRIP, 0, 4 );
  }
}
