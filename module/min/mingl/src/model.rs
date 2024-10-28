mod private
{
  pub use tobj;
  pub use super::obj_model as obj;
}

pub mod obj_model;

crate::mod_interface!
{
  exposed use 
  {
    tobj,
    obj
  };
}