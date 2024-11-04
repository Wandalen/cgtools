/// Internal namespace.
mod private
{
  use crate::*;

  pub fn submit( queue : &web_sys::GpuQueue, buffer : web_sys::GpuCommandBuffer )
  {
    queue.submit( & Vec::from( [ buffer ] ).into() );
  }
}

crate::mod_interface!
{
  own use
  {
    submit
  };
}
