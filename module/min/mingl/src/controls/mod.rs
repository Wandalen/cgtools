/// Provides controllers with different purposes that
/// connect inputs with objects on scene
mod private
{

}

::mod_interface::mod_interface!
{
  /// Provides an orbit-style camera controller.
  #[ cfg( all( feature = "math", feature = "camera_orbit_controls" ) ) ]
  layer camera_orbit_controls;

  /// Provides an character controller.
  #[ cfg( all( feature = "math", feature = "character_controls" ) ) ]
  layer character_controls;
}
