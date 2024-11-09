/// Internal namespace.
mod private
{
    use crate::*;
  pub trait AsWeb< T >
  {
    fn to_web( self ) -> T;
  }

  macro_rules! impl_to_web 
  {
    ( $local:ty, $web:ident ) => 
    {
        impl AsWeb< web_sys::$web > for $local
        {
            fn to_web( self ) -> web_sys::$web
            {
                self.into()
            }
        }
    };
  }

  // Layout
  impl_to_web!( VertexAttribute, GpuVertexAttribute );
  impl_to_web!( VertexBufferLayout, GpuVertexBufferLayout );

  // Descriptor
  impl_to_web!( BindGroupDescriptor< '_ >, GpuBindGroupDescriptor );
  impl_to_web!( RenderPassDescriptor< '_ >, GpuRenderPassDescriptor );
  impl_to_web!( RenderPipelineDescriptor< '_ >, GpuRenderPipelineDescriptor );
  impl_to_web!( SamplerDescriptor< '_ >, GpuSamplerDescriptor );
  impl_to_web!( TextureDescriptor< '_ >, GpuTextureDescriptor );
  impl_to_web!( BindGroupLayoutEntry, GpuBindGroupLayoutEntry );
  impl_to_web!( BindGroupLayoutDescriptor, GpuBindGroupLayoutDescriptor );
  impl_to_web!( PipelineLayoutDescriptor< '_ >, GpuPipelineLayoutDescriptor );

  // Bind group entry
  impl_to_web!( BindGroupEntry, GpuBindGroupEntry );
  impl_to_web!( BufferBinding< '_ >, GpuBufferBinding );

  // State
  impl_to_web!( VertexState< '_ >, GpuVertexState );
  impl_to_web!( FragmentState< '_ >, GpuFragmentState );
  impl_to_web!( DepthStencilState, GpuDepthStencilState );
  impl_to_web!( StencilFaceState, GpuStencilFaceState );
  impl_to_web!( MultiSampleState, GpuMultisampleState );
  impl_to_web!( PrimitiveState, GpuPrimitiveState );
  impl_to_web!( ColorTargetState, GpuColorTargetState );
  impl_to_web!( BlendComponent, GpuBlendComponent );
  impl_to_web!( BlendState, GpuBlendState );

  // Render pass
  impl_to_web!( ColorAttachment< '_ >, GpuRenderPassColorAttachment );
  impl_to_web!( DepthStencilAttachment< '_ >, GpuRenderPassDepthStencilAttachment );

}

crate::mod_interface!
{
  exposed use
  {
    AsWeb
  };
}
