/// Internal namespace.
mod private
{
  use crate::*;

  pub fn submit( device : &web_sys::GpuDevice, buffer : web_sys::GpuCommandBuffer )
  {
    let queue = device.queue();
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
