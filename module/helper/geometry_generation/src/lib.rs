#![ doc = include_str!( "../README.md" ) ]

mod private
{

}

::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;

  layer text;

  layer primitive;

  layer primitive_data;
}