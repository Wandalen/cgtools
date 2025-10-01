mod private
{
  use minwebgl as gl;
  use gl::
  {
    GL,
    F32x4x4,
    web_sys::
    {
      js_sys::Float32Array,
      WebGlTexture,
      WebGlUniformLocation
    }
  };
  use crate::webgl::Node;
  use std::{ cell::RefCell, collections::HashSet, rc::Rc };
  use std::collections::{ HashSet, HashMap };

  /// Global transform matrices texture slot
  pub const GLOBAL_MATRICES_SLOT : u32 = 13;
  /// Inverse bind matrices texture slot
  pub const INVERSE_MATRICES_SLOT : u32 = 14;
  /// Displacements texture slot
  pub const DISPLACEMENTS_SLOT : u32 = 15;

  /// Loads data to data texture where every pixel
  /// is 4 float values. Used for packing matrices array
  fn load_texture_data_4f
  (
    gl : &GL,
    texture : &WebGlTexture,
    data : &[ f32 ],
    size : [ u32; 2 ],
  )
  {
    gl.active_texture( GL::TEXTURE0 );
    gl.bind_texture( GL::TEXTURE_2D, Some( texture ) );

    // Create a Float32Array from the Rust slice
    let js_data = Float32Array::from( data );

    let _ = gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view
    (
      GL::TEXTURE_2D,
      0,
      GL::RGBA32F as i32,
      size[ 0 ] as i32,
      size[ 1 ] as i32,
      0,
      GL::RGBA,
      GL::FLOAT,
      Some( &js_data ),
    );

    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32 );

    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32 );
  }

  /// Binds a texture to a texture unit and uploads its location to a uniform.
  ///
  /// # Arguments
  ///
  /// * `gl` - The WebGL2 rendering context.
  /// * `texture` - The texture to bind.
  /// * `location` - The uniform location in the shader for the sampler.
  /// * `slot` - The texture unit to bind to ( e.g., `GL::TEXTURE0` ).
  fn upload_texture
  (
    gl : &GL,
    texture : &WebGlTexture,
    location : Option< WebGlUniformLocation >,
    slot : u32,
  )
  {
    gl.active_texture( gl::TEXTURE0 + slot );
    gl.bind_texture( GL::TEXTURE_2D, Some( &texture ) );
    // Tell the sampler uniform in the shader which texture unit to use ( 0 for GL_TEXTURE0, 1 for GL_TEXTURE1, etc. )
    gl.uniform1i( location.as_ref(), slot as i32 );
  }

  /// Set of virtual bones used to deform and control the
  /// movement of a 3D models. It's a fundamental concept
  /// in skeletal animation, the most common method for
  /// rigging and animating complex models.
  pub struct Skeleton
  {
    /// List of nodes name that is part of skeleton
    joints : Vec< Rc< RefCell< Node > > >,
    /// List of nodes correcting matrices used in nodes
    /// transform for playing skeletal animations
    _inverse_bind_matrices :  Vec< F32x4x4 >,
    /// Global matrices data texture
    global_texture : WebGlTexture,
    /// Inverse matrices data texture
    inverse_texture : WebGlTexture,
    /// Defines if [`Skeleton`] is recently cloned,
    /// but not all fields have been cloned too
    need_clone_inner : bool,

    /// Morph targets positions displacements
    positions_displacements : Option< Vec< [ f32; 3 ] > >,
    /// Morph targets normals displacements
    normals_displacements : Option< Vec< [ f32; 3 ] > >,
    /// Morph targets tangents displacements
    tangents_displacements : Option< Vec< [ f32; 3 ] > >,
    /// Morph targets displacements texture
    displacements_texture : Option< WebGlTexture >,
    /// Define if need update [`Self::displacements_texture`]
    need_update_displacement : bool,
    /// Displacements count. Must be sum of mesh primitives vertices count
    vertices_count : Option< usize >,
  }

  impl Skeleton
  {
    /// Create a new [`Skeleton`] instance
    ///
    /// # Arguments
    ///
    /// * `gl` - The WebGL2 rendering context.
    /// * `joints` - mapping names to inverse bind matrices of [`Skeleton`] nodes
    ///
    pub fn new
    (
      gl : &GL,
      joints : Vec< ( Rc< RefCell< Node > >, F32x4x4 ) >
    ) -> Option< Self >
    {
      let mut nodes = vec![];
      let mut inverse_bind_matrices = vec![];

      for ( node, matrix ) in joints
      {
        nodes.push( node );
        inverse_bind_matrices.push( matrix );
      }

      let mut inverse_data = inverse_bind_matrices.iter()
      .map
      (
        | m | m.to_array().to_vec()
      )
      .flatten()
      .collect::< Vec< _ > >();

      let a = 4.0_f32.powf( ( inverse_data.len() as f32 ).sqrt().log( 4.0 ).ceil() ) as u32;
      let texture_size = [ a, a ];

      inverse_data.extend( vec![ 0.0; ( a * a * 4 ) as usize - inverse_data.len() ] );

      let inverse_texture = gl.create_texture()?;
      load_texture_data_4f( gl, &inverse_texture, inverse_data.as_slice(), texture_size );

      Some
      (
        Self
        {
          joints : nodes,
          _inverse_bind_matrices : inverse_bind_matrices,
          global_texture : gl.create_texture()?,
          inverse_texture,
          need_clone_inner : false,

          positions_displacements : None,
          normals_displacements : None,
          tangents_displacements : None,
          displacements_texture : None,
          vertices_count : None,
          need_update_displacement : false
        }
      )
    }

    /// Upload inverse bind matrices texture to current shader program
    ///
    /// Displacement texture aligment:
    ///
    /// +---------------------------------...---------------...----------------...--------------...-------+
    /// |                                          Texture row                                            |
    /// +---------------------------------...---------------...----------------...-------+------...-------+
    /// |                       One combined vertex multitarget block                    |      ...       |
    /// +---------------------------------...-------+-------...-------+--------...-------+------...-------+
    /// |               Positions targets           | Normals targets | Tangents targets |      ...       |
    /// +--------------------------+---+--...--+----+----+--...--+----+-----+--...--+----+------...-------+
    /// |        One target        |   |       |    |    |       |    |     |       |    |      ...       |
    /// +-----+--------------+-----+---+--...--+----+----+--...--+----+-----+--...--+----+------...-------+
    /// |  X  | Y (4 bytes)  |  Z  |   |       |    |    |       |    |     |       |    |      ...       |
    /// +-----+--------------+-----+---+--...--+----+----+--...--+----+-----+--...--+----+------...-------+
    ///
    pub fn upload
    (
      &mut self,
      gl : &GL,
      locations : &HashMap< String, Option< gl::WebGlUniformLocation > >
    )
    {
      if self.need_clone_inner
      {
        if let Some( global_texture ) = gl.create_texture()
        {
          self.global_texture = global_texture;
          self.need_clone_inner = false;
        }

        if let Some( displacements_texture ) = gl.create_texture()
        {
          self.displacements_texture = displacements_texture;
          self.need_clone_inner = false;
        }
      }

      if self.need_update_displacement
      {

        self.need_update_displacement = false;
      }

      let global_matrices = self.joints.iter()
      .map
      (
        | node | node.borrow().get_world_matrix()
      )
      .collect::< Vec< _ > >();

      let mut global_data = global_matrices.iter()
      .map
      (
        | m | m.to_array().to_vec()
      )
      .flatten()
      .collect::< Vec< _ > >();

      let a = 4.0_f32.powf( ( global_data.len() as f32 ).sqrt().log( 4.0 ).ceil() ) as u32;
      let texture_size = [ a, a ];

      global_data.extend( vec![ 0.0; ( a * a * 4 ) as usize - global_data.len() ] );

      let global_matrices_loc = locations.get( "globalJointTransformMatrices" ).unwrap();
      let inverse_matrices_loc = locations.get( "inverseBindMatrices" ).unwrap();
      let texture_size_loc = locations.get( "matricesTextureSize" ).unwrap();

      load_texture_data_4f( gl, &self.global_texture, global_data.as_slice(), texture_size );
      upload_texture( gl, &self.global_texture, global_matrices_loc.clone(), GLOBAL_MATRICES_SLOT );
      upload_texture( gl, &self.inverse_texture, inverse_matrices_loc.clone(), INVERSE_MATRICES_SLOT );
      gl::uniform::upload( &gl, texture_size_loc.clone(), texture_size.as_slice() ).unwrap();
    }

    pub fn set_displacement
    (
      &mut self,
      displacement_array : Option< Vec< [ f32; 3 ] > >,
      displacement_type : gltf::Semantic,
      vertices_count : usize
    )
    {
      if Some( vertices_count ) != self.vertices_count && self.vertices_count.is_some()
      {
        return;
      }

      if self.vertices_count.is_none()
      {
        self.vertices_count = Some( vertices_count );
      }

      let positions_len = self.positions_displacements.map( | v | v.len() ).unwrap_or_default();
      let normals_len = self.normals_displacements.map( | v | v.len() ).unwrap_or_default();
      let tangents_len = self.tangents_displacements.map( | v | v.len() ).unwrap_or_default();
      let mut unique = [ displacement_array, positions_len, normals_len, tangents_len ].iter().collect::< HashSet< _ > >();
      unique.remove( &0 );
      if unique.len() > 1
      {
        return;
      }

      match displacement_type
      {
        gltf::Semantic::Positions => { self.positions_displacements = displacement_array; },
        gltf::Semantic::Normals => { self.normals_displacements = displacement_array; },
        gltf::Semantic::Tangents => { self.tangents_displacements = displacement_array; }
        _ => ()
      }
      match displacement_type
      {
        gltf::Semantic::Positions |
        gltf::Semantic::Normals |
        gltf::Semantic::Tangents => { self.need_update_displacement = true; }
        _ => ()
      }
    }
  }

  impl Clone for Skeleton
  {
    fn clone( &self ) -> Self
    {
      Self
      {
        joints : self.joints.iter()
        .map( | n | n.borrow().clone_tree() )
        .collect::< Vec< _ > >(),
        _inverse_bind_matrices : self._inverse_bind_matrices.clone(),
        global_texture : self.global_texture.clone(),
        inverse_texture : self.inverse_texture.clone(),
        need_clone_inner : true,
        positions_displacements : self.positions_displacements.clone(),
        normals_displacements : self.normals_displacements.clone(),
        tangents_displacements : self.tangents_displacements.clone(),
        displacements_texture : self.displacements_texture.clone(),
        need_update_displacement : self.need_update_displacement.clone(),
        vertices_count : self.vertices_count
      }
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    Skeleton,
    GLOBAL_MATRICES_SLOT,
    INVERSE_MATRICES_SLOT,
    DISPLACEMENTS_SLOT
  };
}
