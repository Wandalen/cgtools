//!
//! Error type for all fallible operations in this crate.
//!

use mingl::mod_interface;

mod private
{
  use error_tools::dependency::thiserror;

  /// The error type for all fallible operations exposed by `minvulkan`.
  #[ derive( Debug, thiserror::Error ) ]
  #[ non_exhaustive ]
  pub enum Error
  {
    /// The Vulkan loader library ( e.g. `libvulkan.so.1` ) could not be found or loaded on
    /// this system.
    #[ error( "failed to load the Vulkan loader library : {0}" ) ]
    EntryLoad( ash::LoadingError ),
    /// `vkCreateInstance` failed.
    #[ error( "failed to create a Vulkan instance : {0}" ) ]
    InstanceCreate( ash::vk::Result ),
    /// `vkEnumeratePhysicalDevices` failed.
    #[ error( "failed to enumerate Vulkan physical devices : {0}" ) ]
    PhysicalDeviceEnumerate( ash::vk::Result ),
    /// No enumerated physical device exposes a graphics-capable queue family.
    #[ error( "no Vulkan physical device exposes a graphics-capable queue family" ) ]
    NoSuitableDevice,
    /// `vkCreateDevice` failed.
    #[ error( "failed to create a Vulkan logical device : {0}" ) ]
    DeviceCreate( ash::vk::Result ),
  }
}

mod_interface!
{
  exposed use Error;
}
