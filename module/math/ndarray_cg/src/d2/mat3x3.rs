mod private
{

}

mod transformation;

crate::mod_interface!
{

  own use transformation::
  {
    from_angle_y,
    from_axis_angle
  };

}
