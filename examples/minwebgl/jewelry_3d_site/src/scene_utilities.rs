use std::{ cell::RefCell, rc::Rc };
use minwebgl as gl;
use gl::
{
  GL,
  JsCast,
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

/// Creates template for [`TextureInfo`] with texture without data
#[ inline ]
pub async fn create_empty_texture( gl : &GL ) -> Option< TextureInfo >
{
  // Handle texture creation failure gracefully
  let texture = gl.create_texture()?;

  let sampler = Sampler::former()
  .min_filter( MinFilterMode::Linear )
  .mag_filter( MagFilterMode::Linear )
  .wrap_r( WrappingMode::Repeat )
  .wrap_s( WrappingMode::Repeat )
  .wrap_t( WrappingMode::Repeat )
  .end();

  let texture = Texture::former()
  .target( GL::TEXTURE_2D )
  .source( texture.clone() )
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

/// Finds [`Node`]'s in [`Scene`]. [`Node`] name must be
/// available ( not [`None`] ), contain substring sensitive
/// or not to case relatively to case_sensitive bool
#[ must_use ]
#[ inline ]
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

/// Adds callback and returns bool to signal other program components about [`web_sys::HtmlCanvasElement`] size change
#[ must_use ]
#[ inline ]
pub fn add_resize_callback() -> Option< Rc< RefCell< bool > > >
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

  // Handle event listener registration failure gracefully
  let window = gl::web_sys::window()?;
  window
  .add_event_listener_with_callback("resize", resize_closure.as_ref().unchecked_ref())
  .ok()?;
  resize_closure.forget();

  Some( is_resized )
}
