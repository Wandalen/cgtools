pub struct Context
{
  instance : wgpu::Instance,
  adapter : wgpu::Adapter,
  device : wgpu::Device,
  queue : wgpu::Queue,
}

impl Context
{
  pub fn new_temp() -> Self
  {
    let instance = wgpu::Instance::new
    (
      &wgpu::InstanceDescriptor
      {
        backends : wgpu::Backends::PRIMARY,
        ..Default::default()
      }
    );

    let adapter = minwgpu::helper::request_adapter
    (
      &instance,
      &wgpu::RequestAdapterOptions
      {
        power_preference : wgpu::PowerPreference::HighPerformance,
        ..Default::default()
      }
    ).expect( "Failed to retrieve an adapter" );

    let ( device, queue ) = minwgpu::helper::request_device
    (
      &adapter,
      &wgpu::DeviceDescriptor{
        required_features : wgpu::Features::PUSH_CONSTANTS,
        required_limits : wgpu::Limits
        {
          max_push_constant_size : 16,
          ..Default::default()
        },
        ..Default::default()
      }
    )
    .expect( "Failed to retrieve a device" );

    Self { instance, adapter, device, queue }
  }

  pub fn instance( &self ) -> &wgpu::Instance
  {
    &self.instance
  }

  pub fn adapter( &self ) -> &wgpu::Adapter
  {
    &self.adapter
  }

  pub fn device( &self ) -> &wgpu::Device
  {
    &self.device
  }

  pub fn queue( &self ) -> &wgpu::Queue
  {
    &self.queue
  }
}

impl AsRef< wgpu::Instance > for Context
{
  fn as_ref( &self ) -> &wgpu::Instance
  {
    &self.instance
  }
}

impl AsRef< wgpu::Adapter > for Context
{
  fn as_ref( &self ) -> &wgpu::Adapter
  {
    &self.adapter
  }
}

impl AsRef< wgpu::Device > for Context
{
  fn as_ref( &self ) -> &wgpu::Device
  {
    &self.device
  }
}

impl AsRef< wgpu::Queue > for Context
{
  fn as_ref( &self ) -> &wgpu::Queue
  {
    &self.queue
  }
}
