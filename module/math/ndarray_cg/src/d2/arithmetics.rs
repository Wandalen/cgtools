mod private
{
  // Keeping this commented out as it might be needed in the future
  // use crate::*;
}

mod add;
mod mul;
mod div;

crate::mod_interface!
{
  orphan use
  {
    add::add,
    mul::mul,
    mul::mul_mat_vec,
    div::div_scalar
  };
}
