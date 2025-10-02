mod private
{
  use minwebgl as gl;
  use gl::
  {
    GL,
    F32x4x4,
    I32x3,
    web_sys::
    {
      js_sys::Float32Array,
      WebGlTexture,
      WebGlUniformLocation
    }
  };
  use crate::webgl::Node;
  use std::{ cell::RefCell, rc::Rc };
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
    /// [`Self::displacements_texture`] size
    disp_texture_size : Option< [ u32; 2 ] >,
    /// Morph weights for updating geometry every frame
    morph_weights : Rc< RefCell< Vec< f32 > > >,
    /// Default morph weights
    pub default_weights : Vec< f32 >,
    /// Count of morph targets
    targets_count : Option< usize >,
    /// Offsets of each displacement in `One combined vertex multitarget block`
    /// (see docs of [`Self::upload`]). If offset is -1 it's, means that it
    /// doesn't included into [`Self::displacements_texture`] texture
    disp_offsets : Option< I32x3 >,
    /// Displacements count. Must be sum of mesh primitives vertices count
    vertices_count : Option< usize >,
    /// Define if need update [`Self::displacements_texture`]
    need_update_displacement : bool,
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
          disp_texture_size : None,
          disp_offsets : None,
          targets_count : None,
          vertices_count : None,
          need_update_displacement : false,
          morph_weights : Rc::new( RefCell::new( vec![] ) ),
          default_weights : vec![]
        }
      )
    }

    /// Returns morph weights that is used for updating geometry
    pub fn get_morph_weights( &self ) -> Rc< RefCell< Vec< f32 > > >
    {
      self.morph_weights.clone()
    }

    /// Upload inverse bind matrices texture to current shader program
    ///
    /// Displacement texture aligment:
    ///
    /// +--------------------------------...---------------...----------------...--------------...-------+
    /// |                                         Texture row                                            |
    /// +--------------------------------...---------------...----------------...-------+------...-------+
    /// |                      One combined vertex multitarget block                    |      ...       |
    /// +--------------------------------...-------+-------...-------+--------...-------+------...-------+
    /// |             Positions targets            | Normals targets | Tangents targets |      ...       |
    /// +-------------------------+---+--...--+----+----+--...--+----+-----+--...--+----+------...-------+
    /// |        One target       |   |       |    |    |       |    |     |       |    |      ...       |
    /// +-----+-------------+-----+---+--...--+----+----+--...--+----+-----+--...--+----+------...-------+
    /// |  X  | Y (4 bytes) |  Z  |   |       |    |    |       |    |     |       |    |      ...       |
    /// +-----+-------------+-----+---+--...--+----+----+--...--+----+-----+--...--+----+------...-------+
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

        if self.displacements_texture.is_some()
        {
          if let Some( displacements_texture ) = gl.create_texture()
          {
            self.displacements_texture = Some( displacements_texture );
            self.need_clone_inner = false;
          }
        }
      }

      if self.need_update_displacement
      {
        if self.displacements_texture.is_none()
        {
          if let Some( displacements_texture ) = gl.create_texture()
          {
            self.displacements_texture = Some( displacements_texture );
          }
        }

        let arrays =
        [
          &self.positions_displacements,
          &self.normals_displacements,
          &self.tangents_displacements
        ]
        .iter()
        .filter( | v | v.is_some() )
        .map( | v | v.as_ref().unwrap().clone() )
        .collect::< Vec< _ > >();

        let len = arrays.iter()
        .map( | v | v.len() )
        .max()
        .unwrap_or_default();

        let mut data = ( 0..len )
        .flat_map( | i | arrays.iter().map( move | arr | arr[ i ] ) )
        .flat_map( | t | [ t[ 0 ], t[ 1 ], t[ 2 ], 0.0 ] )
        .collect::< Vec< _ > >();

        let attributes_count = arrays.len();
        let targets_count = self.vertices_count.map( | vc | len / vc ).unwrap_or_default();
        let vertex_displacement_len = attributes_count * targets_count;
        self.targets_count = Some( targets_count );
        if self.morph_weights.borrow().is_empty()
        {
          *self.morph_weights.borrow_mut() = if self.default_weights.len() == targets_count
          {
            self.default_weights.clone()
          }
          else
          {
            vec![ 0.0; targets_count ]
          };
        }

        if vertex_displacement_len != 0
        {
          let v = vertex_displacement_len as f32;

          let mut i = 0;
          while ( v * i as f32 ).powf( 2.0 ) < data.len() as f32
          {
            i += 1;
          }

          let a = ( v * i as f32 ) as u32;
          let b = ( data.len() as f32 / a as f32 ).ceil() as u32;
          self.disp_texture_size = Some( [ a, b ] );
          data.extend( vec![ 0.0; ( a * b * 4 ) as usize - data.len() ] );
          // gl::info!( "DATA: {:?}", data.get( 0..( vertex_displacement_len * 4 ) ) );
          load_texture_data_4f( gl, self.displacements_texture.as_ref().unwrap(), data.as_slice(), [ a, b ] );
        }

        let mut offset = 0 as i32;
        let offsets =
        [
          &self.positions_displacements,
          &self.normals_displacements,
          &self.tangents_displacements
        ]
        .map
        (
          | v |
          {
            if v.is_some()
            {
              let i = offset;
              offset += 1;
              i
            }
            else
            {
              -1
            }
          }
        );

        self.disp_offsets = Some( I32x3::from_array( offsets ) );

        self.need_update_displacement = false;
      }

      //
      gl::info!( "disp_offsets: {:?}", self.disp_offsets );
      //
      gl::info!( "disp_texture_size: {:?}", self.disp_texture_size );
      //
      gl::info!( "displacements_texture: {:?}", self.displacements_texture.is_some() );
      //
      gl::info!( "targets_count: {:?}", self.targets_count );

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
      gl::uniform::upload( gl, texture_size_loc.clone(), texture_size.as_slice() ).unwrap();

      if self.displacements_texture.is_some()
      {
        if let Some( displacements_loc ) = locations.get( "displacements" )
        {
          upload_texture( gl, self.displacements_texture.as_ref().unwrap(), displacements_loc.clone(), DISPLACEMENTS_SLOT );
        }
        if let Some( morph_weights_loc ) = locations.get( "morphWeights" )
        {
          let data = self.morph_weights
          .borrow()
          .get( 0..self.targets_count.unwrap() )
          .map( | v | v.iter().map( | i | [ *i; 1 ] ).collect::< Vec< _ > >() )
          .unwrap_or( vec![ [ 0.0_f32; 1 ]; self.targets_count.unwrap() ] );
          gl::uniform::upload
          (
            gl,
            morph_weights_loc.clone(),
            data.as_slice()
          )
          .unwrap();
          //
      gl::info!( "{:?}", data );
        }
        if let Some( disp_size_loc ) = locations.get( "displacementsTextureSize" )
        {
          gl::uniform::upload( gl, disp_size_loc.clone(), self.disp_texture_size.as_slice() ).unwrap();
        }
        //
      gl::info!( "{:?}", self.disp_texture_size );
        if let Some( targets_count_loc ) = locations.get( "targetsCount" )
        {
          gl::uniform::upload( gl, targets_count_loc.clone(), &( self.targets_count.unwrap() as u32 ) ).unwrap();
        }
        if let Some( disp_offsets_loc ) = locations.get( "displacementsOffsets" )
        {
          gl::uniform::upload( gl, disp_offsets_loc.clone(), &self.disp_offsets.unwrap().to_array()[ .. ] ).unwrap();
        }
      }
    }

    /// Sets one morph targets vertex attribute data that will be packed into texture
    pub fn set_displacement
    (
      &mut self,
      displacement_array : Option< Vec< [ f32; 3 ] > >,
      displacement_type : gltf::Semantic,
      vertices_count : usize
    )
    {
      //
      gl::info!( "{:?}", ( &displacement_type, displacement_array.is_some() ) );
      if Some( vertices_count ) != self.vertices_count && self.vertices_count.is_some()
      {
        return;
      }

      if self.vertices_count.is_none()
      {
        self.vertices_count = Some( vertices_count );
      }

      let positions_len = self.positions_displacements.as_ref().map( | v | v.len() ).unwrap_or_default();
      let normals_len = self.normals_displacements.as_ref().map( | v | v.len() ).unwrap_or_default();
      let tangents_len = self.tangents_displacements.as_ref().map( | v | v.len() ).unwrap_or_default();
      let mut unique =
      [
        displacement_array.as_ref().map( | v | v.len() ).unwrap_or( 0 ),
        positions_len,
        normals_len,
        tangents_len
      ]
      .into_iter()
      .collect::< HashSet< _ > >();
      unique.remove( &0 );
      /*gl::info!
      (
        "Unique: {:?}",
        unique
      );*/
      if unique.len() > 1
      {
        return;
      }

      let displacement_is_some = displacement_array.is_some();

      match displacement_type
      {
        gltf::Semantic::Positions => { self.positions_displacements = displacement_array; },
        gltf::Semantic::Normals => { self.normals_displacements = displacement_array; },
        gltf::Semantic::Tangents => { self.tangents_displacements = displacement_array; }
        _ => ()
      }

      if self.displacements_texture.is_some() || displacement_is_some
      {
        match displacement_type
        {
          gltf::Semantic::Positions |
          gltf::Semantic::Normals |
          gltf::Semantic::Tangents => { self.need_update_displacement = true; }
          _ => ()
        }
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
        vertices_count : self.vertices_count,
        disp_texture_size : self.disp_texture_size.clone(),
        targets_count : self.targets_count.clone(),
        disp_offsets : self.disp_offsets.clone(),
        morph_weights : Rc::new( RefCell::new( self.morph_weights.borrow().clone() ) ),
        default_weights : self.default_weights.clone()
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
