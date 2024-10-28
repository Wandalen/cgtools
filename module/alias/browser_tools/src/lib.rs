#![ doc = include_str!( "../readme.md" ) ]

use ::mod_interface::mod_interface;

mod private
{
}

mod_interface!
{
  reuse ::browser_log;
}
