use web_sys::
{
  wasm_bindgen,
  HtmlImageElement,
};
use wasm_bindgen::{ prelude::*, JsCast };

/// Provide full path to image like `"static/image.png"`
pub fn load_image( path : &str, on_load_callback : Box< dyn Fn( &HtmlImageElement ) > )
{
  let window = web_sys::window().expect( "Should have a window" );
  let document = window.document().expect( "Should have a document" );
  let image = document.create_element( "img" ).unwrap().dyn_into::< HtmlImageElement >().unwrap();
  let img = image.clone();
  let on_load_callback : Closure< dyn Fn() > = Closure::new( move || on_load_callback( &img ) );
  image.set_onload( Some( on_load_callback.as_ref().unchecked_ref() ) );
  on_load_callback.forget();
  let origin = window.location().origin().expect( "Should have an origin" );
  let url = format!( "{origin}/{path}" );
  image.set_src( &url );
}

pub fn get_element_by_id_unchecked< T : JsCast >( id : &str ) -> T
{
  let document = web_sys::window()
  .expect( "Should have a window" )
  .document()
  .expect( "Should have a document" );
  document.get_element_by_id( id )
  .expect( &format!( "No element with id '{id}'" ) )
  .dyn_into::< T >()
  .expect( &format!( "Element is not of type {}", std::any::type_name::< T >() ) )
}
