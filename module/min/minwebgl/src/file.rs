/// Internal namespace.
mod private
{
  use crate::*;

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

}

crate::mod_interface!
{

  own use load;

}
