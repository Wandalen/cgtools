/// Internal namespace.
mod private
{
  use wasm_bindgen::{ prelude::Closure, JsCast };
  use web_sys::HtmlImageElement;
  use crate::web::*;

  // qqq : implement typed errors
  // qqq : documentation, please

  pub async fn load( file_name : &str ) -> Result< Vec< u8 >, JsValue >
  {

    let opts = web_sys::RequestInit::new();
    opts.set_method( "GET" );
    opts.set_mode( web_sys::RequestMode::Cors );

    let window = web_sys::window().unwrap();
    let origin = window.location().origin().unwrap();
    let url = format!( "{}/static/{}", origin, file_name );

    let request = web_sys::Request::new_with_str_and_init( &url, &opts ).expect( "Invalid url" );

    let resp_value = JsFuture::from( window.fetch_with_request( &request ) ).await.expect( "Fetch request fail" );
    let resp : web_sys::Response = resp_value.dyn_into().unwrap();
    let array_buffer_promise = resp.array_buffer()?;
    let array_buffer = JsFuture::from( array_buffer_promise ).await?;

    let uint8_array = js_sys::Uint8Array::new( &array_buffer );
    let mut data = vec![ 0; uint8_array.length() as usize ];
    uint8_array.copy_to( &mut data[ .. ] );

    Ok( data )
  }

  pub fn load_image_with_callback( path : &str, on_load_callback : impl Fn( &HtmlImageElement ) + 'static )
  {
    let window = web_sys::window().expect( "Should have a window" );
    let origin = window.location().origin().expect( "Should have an origin" );
    let url = format!( "{origin}/static/{path}" );

    let document = window.document().expect( "Should have a document" );
    let image = document.create_element( "img" ).unwrap().dyn_into::< HtmlImageElement >().unwrap();
    let img = image.clone();
    let on_load_callback : Closure< dyn Fn() > = Closure::new( move || on_load_callback( &img ) );
    image.set_onload( Some( on_load_callback.as_ref().unchecked_ref() ) );
    on_load_callback.forget();
    image.set_src( &url );
  }

  pub fn load_image_with_mut_callback( path : &str, mut on_load_callback : impl FnMut( &HtmlImageElement ) + 'static )
  {
    let window = web_sys::window().expect( "Should have a window" );
    let origin = window.location().origin().expect( "Should have an origin" );
    let url = format!( "{origin}/static/{path}" );

    let document = window.document().expect( "Should have a document" );
    let image = document.create_element( "img" ).unwrap().dyn_into::< HtmlImageElement >().unwrap();
    let img = image.clone();
    let on_load_callback : Closure< dyn FnMut() > = Closure::new( move || on_load_callback( &img ) );
    image.set_onload( Some( on_load_callback.as_ref().unchecked_ref() ) );
    on_load_callback.forget();
    image.set_src( &url );
  }
}

crate::mod_interface!
{

  own use load;
  own use load_image_with_callback;

}
