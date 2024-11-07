/// Internal namespace.
mod private
{
  use crate::web::*;
  use web_sys::HtmlImageElement;
  use wasm_bindgen::prelude::*;

  /// Creates new `HtmlImageElement` and waits asynchronously until it is loaded.
  /// Unfortunately there's no error message provided by JS API on error occurrence
  /// so in this case returns just `None`.
  /// Assume that `src` will be adjusted to "origin/static/`src`".
  pub async fn load( src : &str ) -> Option< HtmlImageElement >
  {
    let image = HtmlImageElement::new().ok()?;

    // here's js promises are used to get
    // either successful or unsuccessful result of image loading.
    // promise then converted into future and awaited until it's done
    let promise = js_sys::Promise::new
    (
      &mut | resolve, reject |
      {
        let on_load = Closure::< dyn Fn() >::new( move || _ = resolve.call1( &JsValue::NULL, &JsValue::NULL ).unwrap() );
        image.set_onload( Some( on_load.as_ref().unchecked_ref() ) );

        let on_error = Closure::< dyn Fn() >::new( move || _ = reject.call1( &JsValue::NULL, &JsValue::NULL ).unwrap() );
        image.set_onerror( Some( on_error.as_ref().unchecked_ref() ) );

        on_load.forget();
        on_error.forget();
      }
    );

    let window = web_sys::window().expect( "Should have a window" );
    let origin = window.location().origin().expect( "Should have an origin" );
    let src = format!( "{origin}/static/{src}" );
    image.set_src( &src );

    let res = JsFuture::from( promise ).await;
    image.set_onload( None );
    image.set_onerror( None );

    match res
    {
      Ok( _ ) => Some( image ),
      Err( _ ) =>
      {
        image.remove();
        None
      }
    }
  }
}

crate::mod_interface!
{

  own use load;

}
