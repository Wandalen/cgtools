#![ doc = include_str!( "../readme.md" ) ]

mod private
{


}

::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;

  //#[ cfg( feature = "webgl" ) ]
  layer webgl;
}