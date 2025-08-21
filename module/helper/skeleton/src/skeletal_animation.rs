//!

mod private
{
  use bytemuck::cast_slice;
  use minwebgl as gl;

  use gl::F32x4x4;
  use std::{ cell::RefCell, rc::Rc };

  // Scene analog
  pub struct Skeleton
  {
    // Contains global matrix, local matrix and joints, weights attributes
    root : Rc< RefCell< Node > >,
    inverse_bind_matrices :  Vec< F32x4x4 >, //HashMap< String, F32x4x4 >
    offset : usize
  }
}

crate::mod_interface!
{
  orphan use
  {
    Skeleton
  };
}