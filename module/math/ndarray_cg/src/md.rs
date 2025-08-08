//! This module serves as a central interface for accessing and managing data within
//! various structures. It organizes functionalities into distinct layers for clarity and modularity,
//! separating data access from attribute management.

mod private
{
}

crate::mod_interface!
{

  /// Defines traits and methods for accessing the components or elements of data structures.
  layer access;
  /// Defines traits and methods for getting and setting specific attributes or properties.
  layer attribute;

}
