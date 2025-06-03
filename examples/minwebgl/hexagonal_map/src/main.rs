use minwebgl as gl;
use std::collections::HashMap;
use tiles_tools::{ coordinates::{ hexagonal, pixel::Pixel }, geometry };
use hexagonal::Coordinate;
use gl::{ JsCast as _, F32x2, Vector, GL, BufferDescriptor };
use web_sys::{ wasm_bindgen::prelude::*, HtmlCanvasElement, HtmlImageElement, WebGlTexture };
use browser_input::{ mouse::MouseButton, Action, Event, EventType };
use renderer::webgl::{ Geometry, AttributeInfo };

type Axial = Coordinate< hexagonal::Axial, hexagonal::Pointy >;

enum TileValue
{
  Empty
}

fn main() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
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

  let hexagon = create_hexagon_geometry( &gl )?;
  hexagon.bind( &gl );
  let shader = create_shader( &gl )?;
  shader.activate();
  let sprite_sheet_path = "static/kenney_hexagon_pack/Spritesheets/hexagonAll_sheet.png";
  let sprite_sheet = load_sprite_sheet( &gl, &document, sprite_sheet_path );

  gl.viewport( 0, 0, width, height );

  let mut map = HashMap::< Axial, TileValue >::new();
  let mut input = browser_input::Input::new( Some( canvas.dyn_into().unwrap() ) );
  let dpr = dpr as f32;
  let zoom = 0.2;
  let inv_canvas_size = F32x2::new( 1.0 / width as f32, 1.0 / height as f32 );
  let ascpect_scale = F32x2::new( 1.0, width as f32 / height as f32 );

  let update = move | _ |
  {
    input.update_state();

    for Event { event_type, .. } in input.event_queue().iter()
    {
      if let EventType::MouseButton( MouseButton::Main, Action::Press ) = event_type
      {
        let Vector ( [ x, y ] ) = input.pointer_position();

        let position = F32x2::new( x as f32 * dpr, y as f32 * dpr );
        let position = ( position * inv_canvas_size - 0.5 ) * 2.0;
        let position : Pixel = ( position / ( zoom * ascpect_scale ) ).into();
        let coordinate = position.into();
        gl::info!( "{coordinate:?}" );
        map.insert( coordinate, TileValue::Empty );
      }
    }

    input.clear_events();

    for coord in map.keys()
    {
      let axial : Axial = ( *coord ).into();
      let mut position : Pixel = axial.into();
      position.data[ 1 ] = -position.data[ 1 ];
      gl.vertex_attrib2fv_with_f32_array( 1, &position.data );
      shader.uniform_upload( "u_scale", ( zoom * ascpect_scale ).as_slice() );
      hexagon.draw( &gl );
    }

    true
  };
  gl::exec_loop::run( update );

  Ok( () )
}

fn create_hexagon_geometry( gl : &GL ) -> Result< Geometry, minwebgl::WebglError >
{
  let positions = geometry::hexagon_triangles_with_tranform
  (
    gl::math::mat2x2h::rot( 30.0f32.to_radians() )
  );

  let position_buffer = gl::buffer::create( &gl )?;
  gl::buffer::upload( &gl, &position_buffer, positions.as_slice(), GL::STATIC_DRAW );
  let info = AttributeInfo
  {
    slot: 0,
    buffer : position_buffer,
    descriptor : BufferDescriptor::new::< [ f32; 2 ] >(),
    bounding_box: Default::default(),
  };

  let mut geometry = Geometry::new( &gl )?;
  geometry.vertex_count = positions.len() as u32;
  geometry.add_attribute( &gl, "position", info, false )?;

  Ok( geometry )
}

fn create_shader( gl : &GL ) -> Result< gl::shader::Program, minwebgl::WebglError >
{
  let vert = include_str!( "../shaders/main.vert" );
  let frag = include_str!( "../shaders/main.frag" );
  gl::shader::Program::new( gl.clone(), vert, frag )
}

fn load_sprite_sheet( gl : &GL, document : &web_sys::Document, src : &str ) -> Option< WebGlTexture >
{
  let img = document.create_element( "img" )
  .unwrap()
  .dyn_into::< HtmlImageElement >()
  .unwrap();
  img.style().set_property( "display", "none" ).unwrap();

  let texture = gl.create_texture();

  let on_load : Closure< dyn Fn() > = Closure::new
  ({
    let gl = gl.clone();
    let img = img.clone();
    let src = src.to_owned();
    let texture = texture.clone();
    move ||
    {
      gl.bind_texture( gl::TEXTURE_2D, texture.as_ref() );
      gl.pixel_storei( gl::UNPACK_FLIP_Y_WEBGL, 1 );
      gl.tex_image_2d_with_u32_and_u32_and_html_image_element
      (
        gl::TEXTURE_2D,
        0,
        gl::RGBA as i32,
        gl::RGBA,
        gl::UNSIGNED_BYTE,
        &img
      ).expect( "Failed to upload data to texture" );
      gl.pixel_storei( gl::UNPACK_FLIP_Y_WEBGL, 0 );

      gl::texture::d2::filter_nearest( &gl );

      web_sys::Url::revoke_object_url( &src ).unwrap();
      img.remove();
    }
  });

  img.set_onload( Some( on_load.as_ref().unchecked_ref() ) );
  img.set_src( &src );
  on_load.forget();

  texture
}
