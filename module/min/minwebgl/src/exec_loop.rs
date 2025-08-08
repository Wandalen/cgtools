/// Internal namespace.
mod private
{
  // use crate::*;

  use web_sys::
  {
    wasm_bindgen::
    {
      closure::Closure,
    },
  };
  use std::
  {
    cell::RefCell,
    rc::Rc,
  };

  /// Starts a requestAnimationFrame loop that repeatedly calls the provided function.
  pub fn run< F >( mut update_and_draw : F )
  where
    F : 'static + FnMut( f64 ) -> bool,
  {
    let render_loop = Rc::new( RefCell::new( None ) );
    let render_loop_clone = render_loop.clone();

    *render_loop.borrow_mut() = Some( Closure::wrap( Box::new( move | timestamp : f64 |
    {
      // Call the update and draw logic
      let continuing = update_and_draw( timestamp );

      // Request the next frame
      if continuing
      {
        request_animation_frame( render_loop_clone.borrow().as_ref().unwrap() );
      }
    }) as Box< dyn FnMut( f64 ) > ));

    request_animation_frame( render_loop.borrow().as_ref().unwrap() );
  }

  /// Helper function to request animation frame
  /// 
  /// Requests that the browser calls the given closure on the next animation frame.
  pub fn request_animation_frame( f : &Closure< dyn FnMut( f64 ) > )
  {
    use wasm_bindgen::JsCast;
    web_sys::window()
    .unwrap()
    .request_animation_frame( f.as_ref().unchecked_ref() )
    .expect( "should register `requestAnimationFrame` OK" );
  }

}

crate::mod_interface!
{

  own use run;
  orphan use request_animation_frame;

}
