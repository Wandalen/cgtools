//! Implementation of JFA outline

use minwebgl as gl;
use gl::
{
  GL,
  JsValue
};

async fn run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( Default::default() );
  let gl = gl::context::retrieve_or_make()?;
  
  // Define the update and draw logic
  let update_and_draw =
  {
    move | t : f64 |
    {
      true
    }
  };

  // Run the render loop
  gl::exec_loop::run( update_and_draw );

  Ok( () )
}

fn main()
{
  gl::spawn_local( async move { run().await.unwrap() } );
}
