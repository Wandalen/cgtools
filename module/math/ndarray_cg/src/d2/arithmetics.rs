mod private
{
  // Keeping this commented out as it might be needed in the future
  // use crate::*;
}

mod add;
mod mul;

crate::mod_interface!
{
  orphan use
  {
    add::add,
    mul::mul
  };
}
