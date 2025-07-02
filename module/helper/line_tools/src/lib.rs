#![ doc = include_str!( "../readme.md" ) ]

mod private
{

}

::mod_interface::mod_interface!
{
  own use ::mod_interface::mod_interface;

  layer d2;

  layer joins;
  layer caps;

  layer mesh;
  layer program;
}