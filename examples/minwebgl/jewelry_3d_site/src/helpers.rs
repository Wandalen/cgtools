use std::{ cell::RefCell, rc::Rc };
use minwebgl as gl;
use gl::
{
  GL,
  JsCast,
  F32x3,
  web_sys::
  {
    WebGlTexture,
    wasm_bindgen::closure::Closure
  }
};
use std::collections::HashMap;
use renderer::webgl::
{
  Node,
  Scene,
  TextureInfo,
  Texture,
  WrappingMode,
  MinFilterMode,
  MagFilterMode,
  Sampler
};

/// Uploads an image from a URL to a WebGL texture.
///
/// This function creates a new `WebGlTexture` and asynchronously loads an image from the provided URL into it.
/// It uses a `Closure` to handle the `onload` event of an `HtmlImageElement`, ensuring the texture is
/// uploaded only after the image has finished loading.
///
/// # Arguments
///
/// * `gl` - The WebGl2RenderingContext.
/// * `src` - A reference-counted string containing the URL of the image to load.
///
/// # Returns
///
/// A `WebGlTexture` object.
pub fn _upload_texture( gl : &GL, src : &str ) -> WebGlTexture
{
  let window = web_sys::window().expect( "Can't get window" );
  let document =  window.document().expect( "Can't get document" );

  let texture = gl.create_texture().expect( "Failed to create a texture" );

  let img_element = document.create_element( "img" )
  .expect( "Can't create img" )
  .dyn_into::< gl::web_sys::HtmlImageElement >()
  .expect( "Can't convert to gl::web_sys::HtmlImageElement" );
  img_element.style().set_property( "display", "none" ).expect( "Can't set property" );
  let load_texture : Closure< dyn Fn() > = Closure::new
  (
    {
      let gl = gl.clone();
      let img = img_element.clone();
      let texture = texture.clone();
      move ||
      {
        gl::texture::d2::upload_no_flip( &gl, Some( &texture ), &img );
        gl.generate_mipmap( gl::TEXTURE_2D );
        img.remove();
      }
    }
  );

  img_element.set_onload( Some( load_texture.as_ref().unchecked_ref() ) );
  img_element.set_src( &src );
  load_texture.forget();

  texture
}

/// Creates a new `TextureInfo` struct with a texture loaded from a file.
///
/// This function calls `upload_texture` to load an image, sets up a default `Sampler`
/// with linear filtering and repeat wrapping, and then combines them into a `TextureInfo`
/// struct.
///
/// # Arguments
///
/// * `gl` - The WebGl2RenderingContext.
/// * `image_path` - The path to the image file, relative to the `static/` directory.
///
/// # Returns
///
/// An `Option<TextureInfo>` containing the texture data, or `None` if creation fails.
pub fn _create_texture
(
  gl : &GL,
  image_path : &str
) -> Option< TextureInfo >
{
  let image_path = format!( "static/{image_path}" );
  let texture_id = _upload_texture( gl, image_path.as_str() );

  let sampler = Sampler::former()
  .min_filter( MinFilterMode::Linear )
  .mag_filter( MagFilterMode::Linear )
  .wrap_s( WrappingMode::Repeat )
  .wrap_t( WrappingMode::Repeat )
  .end();

  let texture = Texture::former()
  .target( GL::TEXTURE_2D )
  .source( texture_id )
  .sampler( sampler )
  .end();

  let texture_info = TextureInfo
  {
    texture : Rc::new( RefCell::new( texture ) ),
    uv_position : 0,
  };

  Some( texture_info )
}

pub fn filter_nodes( scene : &Rc< RefCell< Scene > >, mut substring : String, case_sensitive : bool ) -> HashMap< String, Rc< RefCell< Node > > >
{
  if !case_sensitive
  {
    substring = substring.to_lowercase();
  }
  let mut filtered = HashMap::new();
  let _ = scene.borrow_mut().traverse
  (
    &mut | node : Rc< RefCell< Node > > |
    {
      if let Some( current_name ) = node.borrow().get_name()
      {
        let modified_current_name = if !case_sensitive
        {
          current_name.to_lowercase()
        }
        else
        {
          current_name.to_string()
        };
        if modified_current_name.contains( &substring )
        {
          filtered.insert( current_name.to_string(), node.clone() );
        }
      }
      Ok( () )
    }
  );
  filtered
}

pub fn get_node( scene : &Rc< RefCell< Scene > >, name : String ) -> Option< Rc< RefCell< Node > > >
{
  let mut target = None;
  let _ = scene.borrow_mut().traverse
  (
    &mut | node : Rc< RefCell< Node > > |
    {
      if target.is_some()
      {
        return Ok( () );
      }
      if let Some( current_name ) = node.borrow().get_name()
      {
        if name == current_name.clone().into_string()
        {
          target = Some( node.clone() );
          return Err( gl::WebglError::Other( "" ) );
        }
      }
      Ok( () )
    }
  );
  target
}

pub fn _remove_node_from_scene( root : &Rc< RefCell< Scene > >, node : &Rc< RefCell< Node > > )
{
  let name = node.borrow().get_name().unwrap();

  let remove_child_ids = root.borrow().children
  .iter()
  .enumerate()
  .filter
  (
    | ( _, n ) |
    {
      if let Some( current_name ) = n.borrow().get_name()
      {
        *current_name.clone().into_string() == *name
      }
      else
      {
        false
      }
    }
  )
  .map( | ( i, _ ) | i )
  .collect::< Vec< _ > >();

  for i in remove_child_ids.iter().rev()
  {
    let _ = root.borrow_mut().children.remove( *i );
  }

  let _ = root.borrow_mut().traverse
  (
    &mut | node : Rc< RefCell< Node > > |
    {
      let remove_child_ids = node.borrow().get_children()
      .iter()
      .enumerate()
      .filter
      (
        | ( _, n ) |
        {
          if let Some( current_name ) = n.borrow().get_name()
          {
            *current_name.clone().into_string() == *name
          }
          else
          {
            false
          }
        }
      )
      .map( | ( i, _ ) | i )
      .collect::< Vec< _ > >();

      for i in remove_child_ids.iter().rev()
      {
        let _ = node.borrow_mut().remove_child( *i );
      }

      Ok( () )
    }
  );
}

pub fn _remove_node_from_node( root : &Rc< RefCell< Node > >, node : &Rc< RefCell< Node > > )
{
  let name = node.borrow().get_name().unwrap();
  let _ = root.borrow_mut().traverse
  (
    &mut | node : Rc< RefCell< Node > > |
    {
      let remove_child_ids = node.borrow().get_children()
      .iter()
      .enumerate()
      .filter
      (
        | ( _, n ) |
        {
          if let Some( current_name ) = n.borrow().get_name()
          {
            *current_name.clone().into_string() == *name
          }
          else
          {
            false
          }
        }
      )
      .map( | ( i, _ ) | i )
      .collect::< Vec< _ > >();

      for i in remove_child_ids.iter().rev()
      {
        let _ = node.borrow_mut().remove_child( *i );
      }

      Ok( () )
    }
  );
}

pub fn add_resize_callback() -> Rc< RefCell< bool > >
{
  let is_resized = Rc::new( RefCell::new( false ) );
  let _is_resized = is_resized.clone();

  let resize_closure =
  Closure::wrap
  (
    Box::new
    (
      move | _ : web_sys::Event |
      {
        *_is_resized.borrow_mut() = true;
      }
    ) as Box< dyn FnMut( _ ) >
  );

  gl::web_sys::window()
  .unwrap()
  .add_event_listener_with_callback("resize", resize_closure.as_ref().unchecked_ref())
  .unwrap();
  resize_closure.forget();

  is_resized
}

pub fn to_decart( radius : f32, theta : f32, phi : f32 ) -> F32x3
{
  let phi = phi.to_radians();
  let theta = theta.to_radians();
  let cos_phi = phi.cos();

  F32x3::from
  (
    [
      radius * cos_phi * theta.cos(),
      radius * phi.sin(),
      radius * cos_phi * theta.sin(),
    ]
  )
}
