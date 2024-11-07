/// Internal namespace.
mod private
{
  use crate::web::*;
  use web_sys::HtmlImageElement;
  use wasm_bindgen::prelude::*;

  pub async fn load( src : &str ) -> Option< HtmlImageElement >
  {
    let image = HtmlImageElement::new().ok()?;
    image.set_src( src );

    let promise = js_sys::Promise::new
    (
      &mut | resolve, reject |
      {
        let on_load = Closure::once( move || resolve.call1( &JsValue::NULL, &JsValue::NULL ).unwrap() );
        image.set_onload( Some( on_load.as_ref().unchecked_ref() ) );

        let on_error = Closure::once( move || reject.call1( &JsValue::NULL, &JsValue::NULL ).unwrap() );
        image.set_onerror( Some( on_error.as_ref().unchecked_ref() ) );

        on_load.forget();
        on_error.forget();
      }
    );

    if let Err( _ ) = JsFuture::from( promise ).await
    {
      image.remove();
      return None;
    }

    Some( image )
  }
}


crate::mod_interface!
{

  own use load;

}
