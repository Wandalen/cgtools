//! This module provides a simple and convenient way to create a main application loop
//! for WebAssembly projects, using the browser's `requestAnimationFrame` API.

/// Internal namespace for implementation details.
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

  /// Starts and runs a render loop that calls the provided closure on each animation frame.
  ///
  /// # Arguments
  /// * `update_and_draw` - A closure that takes a `f64` timestamp and performs all per-frame logic.
  ///   It should return `true` to continue the loop or `false` to stop it.
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

  /// A helper function to make a single `requestAnimationFrame` call.
  ///
  /// # Arguments
  /// * `f` - The closure to be executed on the next animation frame.
  ///
  /// # Panics
  /// Panics if it cannot access the browser's `window` object or if the
  /// `request_animation_frame` call itself fails.
  pub fn request_animation_frame( f : &Closure< dyn FnMut( f64 ) > )
  {
    use wasm_bindgen::JsCast;
    web_sys::window()
    .unwrap()
    .request_animation_frame( f.as_ref().unchecked_ref() )
    .expect( "should register `requestAnimationFrame` OK" );
  }

}

// This macro exposes the public interface of the module.
crate::mod_interface!
{
  /// The main function to start the animation loop.
  own use run;
  /// The helper function to request a single animation frame.
  orphan use request_animation_frame;
}
