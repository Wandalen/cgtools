#![ doc = include_str!( "../README.md" ) ]

mod private
{

}

::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;

  /// Text generation and font processing utilities.
  layer text;

  /// Basic geometric primitive creation.
  layer primitive;

  /// Data structures for primitive attributes and transformations.
  layer primitive_data;
}