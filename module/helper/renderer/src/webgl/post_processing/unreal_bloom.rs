
mod private
{
  use minwebgl as gl;
  use crate::webgl::
  {
    post_processing::Pass, program, ProgramInfo
  };

  const MIPS : usize = 5;

  pub struct UnrealBloomPass
  {
    horizontal_targets : Vec< Option< gl::web_sys::WebGlTexture > >,
    vertical_targets : Vec< Option< gl::web_sys::WebGlTexture > >,
    blur_materials : Vec< ProgramInfo< program::GaussianFilterShader > >,
    composite_material : ProgramInfo< program::UnrealBloomShader >,
    copy_material : ProgramInfo< program::EmptyShader >,
  }

  impl UnrealBloomPass 
  { 
    // Format has to be the same as the input texture
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

      let allocate = | t : Option< &gl::web_sys::WebGlTexture >, width, height |
      {
        gl.bind_texture( gl::TEXTURE_2D, t );
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_array_buffer_view_and_src_offset
        (
          gl::TEXTURE_2D,
          0,
          format as i32,
          width as i32,
          height as i32,
          0,
          gl::RGB,
          gl::FLOAT,
          &gl::js_sys::Float32Array::from( [].as_slice() ).into(),
          0
        ).expect( "Failed to allocate memory for a cube texture" );

        gl.tex_parameteri( gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32 );
      };

      let kernel_radius = [ 3, 5, 7, 9, 11 ];
      
    
      let mut size = [ width / 2, height / 2 ];
      // Generate textures for blurs at different mipmap levels.
      // The blur will go in two passes : image > horizontal > vertical
      for _ in 0..MIPS
      {
        let horizontal = gl.create_texture();
        let vertical = gl.create_texture();

        allocate( horizontal.as_ref(), size[ 0 ], size[ 1 ] );
        allocate( vertical.as_ref(), size[ 0 ], size[ 1 ] );

        horizontal_targets.push( horizontal );
        vertical_targets.push( vertical );

        size[ 0 ] /= 2;
        size[ 1 ] /= 2;
      }

      let vs_shader = include_str!( "../shaders/big_triangle.vert" );
      let fs_shader = include_str!( "../shaders/filters/gaussian.frag" );

      let mut size = [ width / 2, height / 2 ];
      let mut blur_materials = Vec::new();
      for i in 0..MIPS
      {
        let fs_shader = format!( "#define KERNEL_RADIUS {}\n{}", kernel_radius[ i ], fs_shader );
        let blur_material = gl::ProgramFromSources::new( vs_shader, &fs_shader ).compile_and_link( gl )?;
        let blur_material = ProgramInfo::< program::GaussianFilterShader >::new( gl, blur_material );

        let locations = blur_material.get_locations();
        let coefficients = get_gaussian_coefficients( kernel_radius[ i ] );
        let inv_size = [ 1.0 / size[ 0 ] as f32, 1.0 / size[ 1 ] as f32 ];
        blur_material.bind( gl );
        gl.uniform1fv_with_f32_array( locations.get( "kernel" ).unwrap().as_ref(), coefficients.as_slice() );
        gl::uniform::upload( gl, locations.get( "invSize" ).unwrap().clone(), &inv_size[ .. ] )?;

        blur_materials.push( blur_material );

        size[ 0 ] /= 2;
        size[ 1 ] /= 2;
      }

      let fs_shader = include_str!( "../shaders/post_processing/unreal_bloom.frag" );
      let fs_shader = format!( "#define NUM_MIPS {}\n{}", MIPS, fs_shader );
      let composite_material = gl::ProgramFromSources::new( vs_shader, &fs_shader ).compile_and_link( gl )?;
      let composite_material = ProgramInfo::< program::UnrealBloomShader >::new( gl, composite_material );

      const BLOOM_FACTORS : [ f32; 5 ] = [ 1.0, 0.8, 0.6, 0.4, 0.2 ];
      const BLOOM_TINT : [ [ f32; 3 ]; 5 ] = [ [ 1.0; 3 ]; 5 ];
      let locations = composite_material.get_locations();
      composite_material.bind( gl );
      
      gl.uniform1fv_with_f32_array( locations.get( "bloomFactors" ).unwrap().as_ref(), &BLOOM_FACTORS[ .. ] );
      gl.uniform3fv_with_f32_array( locations.get( "bloomTintColors" ).unwrap().as_ref(), BLOOM_TINT.as_flattened() );
      gl.uniform1i( locations.get( "blurTexture0" ).unwrap().clone().as_ref() , 0 );
      gl.uniform1i( locations.get( "blurTexture1" ).unwrap().clone().as_ref() , 1 );
      gl.uniform1i( locations.get( "blurTexture2" ).unwrap().clone().as_ref() , 2 );
      gl.uniform1i( locations.get( "blurTexture3" ).unwrap().clone().as_ref() , 3 );
      gl.uniform1i( locations.get( "blurTexture4" ).unwrap().clone().as_ref() , 4 );
      for i in 0..MIPS
      {
        gl.active_texture( gl::TEXTURE0 + i as u32 );
        gl.bind_texture( gl::TEXTURE_2D, vertical_targets[ i ].as_ref() );
      }

      let fs_shader = include_str!( "../shaders/copy.frag" );

      let copy_material = gl::ProgramFromSources::new( vs_shader, fs_shader ).compile_and_link( gl )?;
      let copy_material = ProgramInfo::< program::EmptyShader >::new( copy_material );

      Ok
      (
        Self 
        { 
          horizontal_targets,
          vertical_targets,
          blur_materials,
          composite_material,
          copy_material
        }
      )
    }  
  }

  impl Pass for UnrealBloomPass
  {
    fn render
    (
      &self,
      gl : &gl::WebGl2RenderingContext,
      input_texture : Option< &gl::web_sys::WebGlTexture >
    ) -> Result< Option< gl::web_sys::WebGlTexture >, gl::WebglError >
    {
      let mut input_texture = input_texture;

      for i in 0..MIPS
      {
        let mat = &self.blur_materials[ i ];
        mat.bind( gl );
        let locations = mat.get_locations();

        // Horizontal blue
        gl::uniform::upload( gl, locations.get( "blurDir" ).unwrap().clone(), gl::F32x2::X.as_slice() )?;
        gl.bind_texture( gl::TEXTURE_2D, input_texture );
        gl.framebuffer_texture_2d
        ( 
          gl::FRAMEBUFFER, 
          gl::COLOR_ATTACHMENT0, 
          gl::TEXTURE_2D, 
          self.horizontal_targets[ i ].as_ref(), 
          0
        );
        gl.draw_arrays( gl::TRIANGLES, 0, 3 );

        // Vertical blur
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
        gl.draw_arrays( gl::TRIANGLES, 0, 3 );

        input_texture = self.vertical_targets[ i ].as_ref();
      }


      let locations = self.composite_material.get_locations();
      self.composite_material.bind( gl );
      gl::uniform::upload( gl, locations.get( "bloomStrength" ).unwrap().clone(), &1.5 )?;
      gl::uniform::upload( gl, locations.get( "bloomRadius" ).unwrap().clone(), &0.4 )?;
      gl.framebuffer_texture_2d
      ( 
        gl::FRAMEBUFFER, 
        gl::COLOR_ATTACHMENT0, 
        gl::TEXTURE_2D, 
        self.horizontal_targets[ 0 ].as_ref(), 
        0
      );
      gl.draw_arrays( gl::TRIANGLES, 0, 3 );

    

      Ok( None )
    }
  }

  fn get_gaussian_coefficients( radius : usize ) -> Vec< f32 >
  {
    let mut c = Vec::with_capacity( radius );

    let sigma = radius as f32;
    for i in 0..radius
    {
      let k = ( -0.5 * ( i as f32 ).powi( 2 ) / sigma.powi( 2 ) ).exp() / ( sigma * ( std::f32::consts::PI * 2.0 ).sqrt() );
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