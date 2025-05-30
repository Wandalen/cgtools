
mod private
{
  use minwebgl as gl;
  use crate::webgl::
  {
    post_processing::{ Pass, VS_TRIANGLE }, program, ProgramInfo
  };

  // Defines the number of mipmap levels to use for the blur effect.
  const MIPS : usize = 5;

  /// Implements an Unreal Bloom post-processing effect from here:
  /// https://github.com/mrdoob/three.js/blob/master/examples/jsm/postprocessing/UnrealBloomPass.js
  ///
  /// This pass blurs the image it takes as input
  pub struct UnrealBloomPass
  {
    /// Intermediate targets for the horizontal blur passes at different mipmap levels.
    horizontal_targets : Vec< Option< gl::web_sys::WebGlTexture > >,
    /// Intermediate targets for the vertical blur passes at different mipmap levels.
    vertical_targets : Vec< Option< gl::web_sys::WebGlTexture > >,
    /// A collection of `GaussianFilterShader` for blurring. There's one program for
    /// each mip level, with different kernel radii.
    blur_materials : Vec< ProgramInfo< program::GaussianFilterShader > >,
    /// Composites all the blurred mipmap levels together to create the final bloom effect.
    composite_material : ProgramInfo< program::UnrealBloomShader >,
    copy_material : ProgramInfo< program::EmptyShader >,
    width : u32,
    height : u32
  }

  impl UnrealBloomPass 
  { 
    /// Creates a new `UnrealBloomPass` instance, initializing all the
    /// necessary WebGL resources for the bloom effect.
    ///
    /// This involves creating multiple textures at decreasing resolutions (mip levels)
    /// for the blur passes, compiling the Gaussian blur and Unreal Bloom shaders,
    /// and setting up their initial uniform values.
    ///
    /// # Arguments
    ///
    /// * `gl` - A reference to the WebGl2RenderingContext.
    /// * `width` - The initial width of the input texture for the bloom pass.
    /// * `height` - The initial height of the input texture for the bloom pass.
    /// * `format` - The internal format of the textures to be created (e.g., `gl::RGBA16F`).
    ///   This should match the format of the input texture.
    pub fn new
    ( 
      gl : &gl::WebGl2RenderingContext, 
      width : u32,
      height : u32,
      format : u32
    ) -> Result< Self, gl::WebglError >
    {
      let mut horizontal_targets = Vec::new();
      let mut vertical_targets = Vec::new();

      // Helper closure to allocate and configure a 2D texture.
      let allocate = | t : Option< &gl::web_sys::WebGlTexture >, width, height |
      {
        gl.bind_texture( gl::TEXTURE_2D, t );
        gl.tex_storage_2d( gl::TEXTURE_2D, 1, format, width as i32, height as i32 );
        gl::texture::d2::filter_linear( gl );
        gl::texture::d2::wrap_clamp( gl );
      };

      // Define the kernel radii for the Gaussian blur at each mip level.
      let kernel_radius = [ 3, 5, 7, 9, 11 ];
      
      // Start with half resolution for the first mip.
      // Generate textures for blur passes at different mipmap levels.
      // The blur process will typically involve two passes: horizontal then vertical.
      let mut size = [ width / 2, height / 2 ];

      for _ in 0..MIPS
      {
        let horizontal = gl.create_texture();
        let vertical = gl.create_texture();

        // Allocate storage for both horizontal and vertical blur textures at the current mip size.
        allocate( horizontal.as_ref(), size[ 0 ], size[ 1 ] );
        allocate( vertical.as_ref(), size[ 0 ], size[ 1 ] );

        horizontal_targets.push( horizontal );
        vertical_targets.push( vertical );

        // Halve the size for the next mip level.
        size[ 0 ] /= 2;
        size[ 1 ] /= 2;
      }

      // Load Gaussian fragment shader source.
      let fs_shader = include_str!( "../shaders/filters/gaussian.frag" );

      let mut size = [ width / 2, height / 2 ];
      //let mut size = [ width, height ];
      let mut blur_materials = Vec::new();

      // Compile and configure a Gaussian blur shader for each mip level.
      for i in 0..MIPS
      {
        // Dynamically inject the KERNEL_RADIUS define into the shader for the current mip.
        let fs_shader = format!( "#version 300 es\n#define KERNEL_RADIUS {}\n{}", kernel_radius[ i ], fs_shader );
        let blur_material = gl::ProgramFromSources::new( VS_TRIANGLE, &fs_shader ).compile_and_link( gl )?;
        let blur_material = ProgramInfo::< program::GaussianFilterShader >::new( gl, blur_material );

        let locations = blur_material.get_locations();
        // Calculate Gaussian coefficients based on the kernel radius.
        let coefficients = get_gaussian_coefficients( kernel_radius[ i ] );
        let inv_size = [ 1.0 / size[ 0 ] as f32, 1.0 / size[ 1 ] as f32 ];
        blur_material.bind( gl );
        gl.uniform1fv_with_f32_array( locations.get( "kernel" ).unwrap().as_ref(), coefficients.as_slice() );
        gl::uniform::upload( gl, locations.get( "invSize" ).unwrap().clone(), &inv_size[ .. ] )?;

        blur_materials.push( blur_material );

        // Update size for the next mip.
        size[ 0 ] /= 2;
        size[ 1 ] /= 2;
      }

      // --- Setup Composite Material ---
      let fs_shader = include_str!( "../shaders/post_processing/unreal_bloom.frag" );
      // Dynamically inject the NUM_MIPS define into the bloom composite shader.
      let fs_shader = format!( "#version 300 es\n#define NUM_MIPS {}\n{}", MIPS, fs_shader );
      let composite_material = gl::ProgramFromSources::new( VS_TRIANGLE, &fs_shader ).compile_and_link( gl )?;
      let composite_material = ProgramInfo::< program::UnrealBloomShader >::new( gl, composite_material );

      // Define bloom factors and tint colors for each mip level.
      const BLOOM_FACTORS : [ f32; 5 ] = [ 1.0, 0.8, 0.6, 0.4, 0.2 ];
      const BLOOM_TINT : [ [ f32; 3 ]; 5 ] = [ [ 1.0; 3 ]; 5 ];
      let locations = composite_material.get_locations();
      composite_material.bind( gl );
      
      gl.uniform1fv_with_f32_array( locations.get( "bloomFactors" ).unwrap().as_ref(), &BLOOM_FACTORS[ .. ] );
      gl.uniform3fv_with_f32_array( locations.get( "bloomTintColors" ).unwrap().as_ref(), BLOOM_TINT.as_flattened() );
      // Assign texture units to the blur textures.
      gl.uniform1i( locations.get( "blurTexture0" ).unwrap().clone().as_ref() , 0 );
      gl.uniform1i( locations.get( "blurTexture1" ).unwrap().clone().as_ref() , 1 );
      gl.uniform1i( locations.get( "blurTexture2" ).unwrap().clone().as_ref() , 2 );
      gl.uniform1i( locations.get( "blurTexture3" ).unwrap().clone().as_ref() , 3 );
      gl.uniform1i( locations.get( "blurTexture4" ).unwrap().clone().as_ref() , 4 );

      let fs_shader = include_str!( "../shaders/copy.frag" );

      let copy_material = gl::ProgramFromSources::new( VS_TRIANGLE, fs_shader ).compile_and_link( gl )?;
      let copy_material = ProgramInfo::< program::EmptyShader >::new( copy_material );

      Ok
      (
        Self 
        { 
          horizontal_targets,
          vertical_targets,
          blur_materials,
          composite_material,
          copy_material,
          width,
          height
        }
      )
    }  
  }

  impl Pass for UnrealBloomPass
  {
    fn renders_to_input( &self ) -> bool 
    {
      false
    }

    fn render
    (
      &self,
      gl : &gl::WebGl2RenderingContext,
      input_texture : Option< gl::web_sys::WebGlTexture >,
      output_texture : Option< minwebgl::web_sys::WebGlTexture >
    ) -> Result< Option< gl::web_sys::WebGlTexture >, gl::WebglError >
    {
      gl.clear_color( 0.0, 0.0, 0.0, 1.0 );
      // --- Multi-Pass Gaussian Blur ---
      // Iterate through mip levels to apply horizontal and vertical Gaussian blur.
      let mut blur_input = input_texture.as_ref();
      let mut size = [ self.width / 2, self.height / 2 ];
      for i in 0..MIPS
      {
        gl.viewport( 0, 0, size[ 0 ] as i32, size[ 1 ] as i32 );

        let mat = &self.blur_materials[ i ];
        mat.bind( gl );
        let locations = mat.get_locations();

        gl.active_texture( gl::TEXTURE0 );

        // Horizontal blur pass:
        gl::uniform::upload( gl, locations.get( "blurDir" ).unwrap().clone(), gl::F32x2::X.as_slice() )?;
        gl.bind_texture( gl::TEXTURE_2D, blur_input );
        gl.framebuffer_texture_2d
        ( 
          gl::FRAMEBUFFER, 
          gl::COLOR_ATTACHMENT0, 
          gl::TEXTURE_2D, 
          self.horizontal_targets[ i ].as_ref(), 
          0
        );
        gl.clear( gl::COLOR_BUFFER_BIT );
        gl.draw_arrays( gl::TRIANGLES, 0, 3 );

        // Vertical blur pass:
        gl::uniform::upload( gl, locations.get( "blurDir" ).unwrap().clone(), gl::F32x2::Y.as_slice() )?;
        gl.bind_texture( gl::TEXTURE_2D, self.horizontal_targets[ i ].as_ref() );
        gl.framebuffer_texture_2d
        ( 
          gl::FRAMEBUFFER, 
          gl::COLOR_ATTACHMENT0, 
          gl::TEXTURE_2D, 
          self.vertical_targets[ i ].as_ref(), 
          0
        );
        gl.clear( gl::COLOR_BUFFER_BIT );
        gl.draw_arrays( gl::TRIANGLES, 0, 3 );

        // gl.bind_texture( gl::TEXTURE_2D, None );
        // gl.framebuffer_texture_2d( gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, gl::TEXTURE_2D, None, 0 );

        blur_input = self.vertical_targets[ i ].as_ref();
        // Update size for the next mip.
        size[ 0 ] /= 2;
        size[ 1 ] /= 2;
      }

      // --- Bloom Composite Pass ---
      gl.viewport( 0, 0, self.width as i32, self.height as i32 );
      let locations = self.composite_material.get_locations();
      self.composite_material.bind( gl );
      for i in 0..MIPS
      {
        gl.active_texture( gl::TEXTURE0 + i as u32 );
        gl.bind_texture( gl::TEXTURE_2D, self.vertical_targets[ i ].as_ref() );
      }
      gl::uniform::upload( gl, locations.get( "bloomStrength" ).unwrap().clone(), &1.5 )?;
      gl::uniform::upload( gl, locations.get( "bloomRadius" ).unwrap().clone(), &0.4 )?;
      gl.framebuffer_texture_2d
      ( 
        gl::FRAMEBUFFER, 
        gl::COLOR_ATTACHMENT0, 
        gl::TEXTURE_2D, 
        output_texture.as_ref(), 
        0
      );
      gl.clear( gl::COLOR_BUFFER_BIT );
      gl.draw_arrays( gl::TRIANGLES, 0, 3 );

      // self.copy_material.bind( gl );
      // gl.blend_func( gl::ONE, gl::ONE );
      // gl.active_texture( gl::TEXTURE0 );
      // gl.bind_texture( gl::TEXTURE_2D, self.horizontal_targets[ 0 ].as_ref() );
      // gl.framebuffer_texture_2d
      // ( 
      //   gl::FRAMEBUFFER, 
      //   gl::COLOR_ATTACHMENT0, 
      //   gl::TEXTURE_2D, 
      //   output_texture.as_ref(), 
      //   0
      // );
      // gl.draw_arrays( gl::TRIANGLES, 0, 3 );

      // Unbind the attachment
      for i in 0..MIPS
      {
        gl.active_texture( gl::TEXTURE0 + i as u32 );
        gl.bind_texture( gl::TEXTURE_2D, None );
      }

      gl.framebuffer_texture_2d
      (
        gl::FRAMEBUFFER, 
        gl::COLOR_ATTACHMENT0, 
        gl::TEXTURE_2D, 
        None, 
        0
      );

      Ok( output_texture )
    }
  }

  /// Calculates Gaussian blur coefficients (weights) for a given kernel radius.
  ///
  /// # Arguments
  ///
  /// * `radius` - The radius of the Gaussian kernel (e.g., 3 means a 7x7 kernel).
  fn get_gaussian_coefficients( radius : usize ) -> Vec< f32 >
  {
    let mut c = Vec::with_capacity( radius );

    let sigma = radius as f32;
    for i in 0..radius
    {
      let e = ( -0.5 * ( i as f32 ).powi( 2 ) / sigma.powi( 2 ) ).exp();
      let denom = sigma * ( std::f32::consts::PI * 2.0 ).sqrt();
      let k = e / denom;
      c.push( k );
    }

    c
  }
}

crate::mod_interface!
{
  orphan use
  {
    UnrealBloomPass
  };
}