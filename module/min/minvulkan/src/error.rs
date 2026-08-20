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
    /// The window declined to hand out a raw window or display handle.
    #[ error( "failed to obtain a raw window handle : {0}" ) ]
    WindowHandle( raw_window_handle::HandleError ),
    /// The platform's `vkCreate*SurfaceKHR` failed, or the platform has no
    /// surface extension at all.
    #[ error( "failed to create a Vulkan window surface : {0}" ) ]
    SurfaceCreate( ash::vk::Result ),
    /// A `vkGetPhysicalDeviceSurface*` query failed.
    #[ error( "failed to query Vulkan surface capabilities : {0}" ) ]
    SurfaceCapabilities( ash::vk::Result ),
    /// The surface reported no supported formats at all.
    #[ error( "the Vulkan surface reports no supported formats" ) ]
    NoSurfaceFormat,
    /// No enumerated physical device exposes a queue family that is both
    /// graphics-capable and able to present to the surface, with
    /// `VK_KHR_swapchain` available.
    #[ error( "no Vulkan physical device can both render and present to this surface" ) ]
    NoPresentDevice,
    /// `vkCreateSwapchainKHR`, or the fence backing acquisition, failed.
    #[ error( "failed to create a Vulkan swapchain : {0}" ) ]
    SwapchainCreate( ash::vk::Result ),
    /// The surface's drawable area has a zero dimension, which no swapchain
    /// can cover — the window is minimized, or not yet mapped.
    ///
    /// Transient and expected rather than a failure : the existing swapchain is
    /// left untouched, so rendering resumes once the window returns.
    #[ error( "the Vulkan surface's drawable area is zero-sized ( {0} x {1} ) -- the window is likely minimized" ) ]
    ZeroExtent( u32, u32 ),
    /// Retrieving the swapchain's images, or creating a view of one, failed.
    #[ error( "failed to retrieve Vulkan swapchain images : {0}" ) ]
    SwapchainImages( ash::vk::Result ),
    /// `vkAcquireNextImageKHR` failed for a reason other than an out-of-date
    /// chain, which is reported as `Frame::Reconfigure` instead.
    #[ error( "failed to acquire a Vulkan swapchain image : {0}" ) ]
    SwapchainAcquire( ash::vk::Result ),
    /// `vkQueuePresentKHR` failed for a reason other than an out-of-date chain,
    /// which is reported as "rebuild the chain" instead.
    #[ error( "failed to present a Vulkan swapchain image : {0}" ) ]
    SwapchainPresent( ash::vk::Result ),
    /// `vkDeviceWaitIdle` failed while retiring resources that in-flight work
    /// may still reference.
    #[ error( "failed to wait for the Vulkan device to go idle : {0}" ) ]
    DeviceWaitIdle( ash::vk::Result ),
  }
}

mod_interface!
{
  exposed use Error;
}
