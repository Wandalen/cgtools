/// Internal namespace.
mod private
{
  use wasm_bindgen::JsCast;
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

  pub async fn load_media< T, F >( path : &str, init_element : F ) -> Result< T, JsValue >
  where
    T : JsCast + AsRef< web_sys::HtmlElement >,
    F : FnOnce( &web_sys::Document ) -> Result< T, JsValue >,
  {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let origin = window.location().origin().unwrap();
    let url = format!( "{}/{}", origin, path );

    let element = init_element( &document )?;

    let load_promise = js_sys::Promise::new
    (
      &mut | resolve, reject |
      {
        let on_event = wasm_bindgen::prelude::Closure::once_into_js
        (
          move ||
          {
            resolve.call0( &JsValue::NULL ).unwrap();
          }
        );

        let on_error = wasm_bindgen::prelude::Closure::once_into_js
        (
          move ||
          {
            reject.call1( &JsValue::NULL, &JsValue::from( "Failed to load media" ) ).unwrap();
          }
        );

        if let Some( image_element ) = element.dyn_ref::< web_sys::HtmlImageElement >()
        {
          image_element.set_src( &url );

          image_element.set_onload( Some( on_event.as_ref().unchecked_ref() ) );
          image_element.set_onerror( Some( on_error.as_ref().unchecked_ref() ) );
        }
        else if let Some( video_element ) = element.dyn_ref::< web_sys::HtmlVideoElement >()
        {
          video_element.set_src( &url );
          let _ = video_element.play().unwrap();

          video_element.set_oncanplay( Some( on_event.as_ref().unchecked_ref() ) );
          video_element.set_onerror( Some( on_error.as_ref().unchecked_ref() ) );
        }
      }
    );

    JsFuture::from( load_promise ).await?;

    Ok( element )
  }
}

crate::mod_interface!
{

  own use load;
  own use load_media;

}
