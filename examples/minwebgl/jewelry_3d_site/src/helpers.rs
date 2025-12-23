use std::{ cell::RefCell, rc::Rc };
use minwebgl as gl;
use gl::
{
  texture::d2::upload_image_by_path,
  GL,
  JsCast,
  F32x3,
  web_sys::wasm_bindgen::closure::Closure
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

pub async fn create_empty_texture( gl : &GL ) -> Option< TextureInfo >
{
  let texture = gl.create_texture();

  let sampler = Sampler::former()
  .min_filter( MinFilterMode::Linear )
  .mag_filter( MagFilterMode::Linear )
  .wrap_r( WrappingMode::Repeat )
  .wrap_s( WrappingMode::Repeat )
  .wrap_t( WrappingMode::Repeat )
  .end();

  let texture = Texture::former()
  .target( GL::TEXTURE_2D )
  .source( texture.clone().unwrap() )
  .sampler( sampler )
  .end();

  let texture = Some
  (
    TextureInfo
    {
      texture : Rc::new( RefCell::new( texture ) ),
      uv_position : 0,
    }
  );

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
pub fn create_texture
(
  gl : &GL,
  image_path : &str
) -> Option< TextureInfo >
{
  let image_path = format!( "static/{image_path}" );
  let texture_id = upload_image_by_path( gl, &image_path );

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

/// Finds [`Node`]'s in [`Scene`]. [`Node`] name must be
/// available ( not [`None`] ), contain substring sensitive
/// or not to case relatively to case_sensitive bool
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
