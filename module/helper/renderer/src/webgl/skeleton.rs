mod private
{
  use minwebgl::{self as gl, WebglError};
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
  use rustc_hash::{ FxHashSet, FxHashMap };

  /// Global transform matrices texture slot
  pub const GLOBAL_MATRICES_SLOT : u32 = 13;
  /// Inverse bind matrices texture slot
  pub const INVERSE_MATRICES_SLOT : u32 = 14;
  /// Displacements texture slot
  pub const DISPLACEMENTS_SLOT : u32 = 15;
  /// Max morph targets support
  pub const MAX_MORPH_TARGETS : usize = 100;

  /// Loads data to data texture where every pixel
  /// is 4 float values. Used for packing matrices array
  ///
  /// # Errors
  ///
  /// Returns `WebglError` if the texture allocation or upload fails.
  pub fn texture_data_4f_load
  (
    gl : &GL,
    texture : &WebGlTexture,
    data : &[ f32 ],
    size : [ u32; 2 ],
  )
  -> Result< (), WebglError >
  {
    gl.active_texture( GL::TEXTURE0 );
    gl.bind_texture( GL::TEXTURE_2D, Some( texture ) );

    // Create a Float32Array from the Rust slice
    let js_data = Float32Array::from( data );

    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_array_buffer_view_and_src_offset
    (
      GL::TEXTURE_2D,
      0,
      GL::RGBA32F as i32,
      size[ 0 ] as i32,
      size[ 1 ] as i32,
      0,
      GL::RGBA,
      GL::FLOAT,
      &js_data,
      0
    )
    .map_err( | _ | WebglError::Other( "Can't write to data texture" ) )?;

    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, GL::NEAREST as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::NEAREST as i32 );

    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32 );
    gl.tex_parameteri( GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32 );

    Ok( () )
  }

  /// Binds a texture to a texture unit and uploads its location to a uniform.
  ///
  /// # Arguments
  ///
  /// * `gl` - The WebGL2 rendering context.
  /// * `texture` - The texture to bind.
  /// * `location` - The uniform location in the shader for the sampler.
  /// * `slot` - The texture unit to bind to ( e.g., `GL::TEXTURE0` ).
  fn texture_upload
  (
    gl : &GL,
    texture : &WebGlTexture,
    location : Option< &WebGlUniformLocation >,
    slot : u32,
  )
  {
    gl.active_texture( gl::TEXTURE0 + slot );
    gl.bind_texture( GL::TEXTURE_2D, Some( texture ) );
    // Tell the sampler uniform in the shader which texture unit to use ( 0 for GL_TEXTURE0, 1 for GL_TEXTURE1, etc. )
    gl.uniform1i( location, slot as i32 );
  }

  /// Nodes' global transform data texture and inverse bind matrices texture
  /// sizes are calculated using this function in the following way:
  /// 1. Get a theoretical square with sides of rational non integer length filled with data.
  /// 2. Then apply log and power to make the texture resolution to be always a multiple of 4.
  ///    We need this to ensure that a single matrix inside of the texture can't be split
  ///    between two rows and all matrices have grid alignment.
  /// 3. The ceil is needed to get the smallest integer side length that fits all the data.
  #[ must_use ]
  pub fn data_texture_size_calculate( data_size : usize ) -> u32
  {
    4.0_f32.powf( ( data_size as f32 ).sqrt().log( 4.0 ).ceil() ) as u32
  }

  /// Computes the `( width, height )`, in texels, of a displacement texture holding
  /// `data_len` floats, keeping each row a whole multiple of `vertex_displacement_len`
  /// ( texels-per-vertex, i.e. `attributes_count * targets_count` ) so a single vertex's
  /// texel block never spans two rows.
  #[ must_use ]
  pub fn displacement_texture_size_compute( data_len : usize, vertex_displacement_len : usize ) -> ( u32, u32 )
  {
    let v = vertex_displacement_len as f32;
    // Fix(BUG-252): plain `floor()` could round `i` down to `0` whenever
    // `sqrt(data_len) < v` (small vertex counts relative to attributes*targets), collapsing
    // the texture width `a` to `0` and forcing `b = data_len / 0 == +inf`, which saturates
    // to `u32::MAX` and always exceeds the caller's size-limit check -- so the update
    // silently and permanently failed every frame with a misleading "texture too large"
    // error instead of ever writing real displacement data.
    // Root cause: rounding the row width down to the nearest multiple of
    // `vertex_displacement_len` can legitimately round all the way down to zero multiples
    // when `data_len` is small; `.max( 1.0 )` guarantees at least one.
    // Pitfall: `a.max( b ) > max_size` looks like it only guards against oversized
    // textures, so it silently absorbed this unrelated div-by-zero-by-`a` failure mode too.
    let i = ( ( data_len as f32 ).sqrt() / v ).floor().max( 1.0 );
    let a = ( v * i ) as u32;
    let b = ( data_len as f32 / a as f32 ).ceil() as u32;
    ( a, b )
  }

  /// Skin joints transforms related data
  #[ derive( Debug ) ]
  pub struct TransformsData
  {
    /// List of nodes name that is part of skeleton
    joints : Vec< Rc< RefCell< Node > > >,
    /// List of nodes correcting matrices used in nodes
    /// transform for playing skeletal animations
    inverse_bind_matrices : Vec< F32x4x4 >,
    /// Global matrices data texture
    global_texture : Option< WebGlTexture >,
    /// Inverse matrices data texture
    inverse_texture : Option< WebGlTexture >,
    /// Define if need update [`Self::inverse_texture`]
    need_update_inverse : bool,
    /// Defines if [`TransformsData`] is recently cloned,
    /// but not all fields have been cloned too
    need_clone_inner : bool,
    /// GL context `global_texture`/`inverse_texture` were allocated from -- retained so
    /// `impl Drop` can free them. `None` until the first `upload()` call actually allocates
    /// a texture ( `new()` receives no `gl` parameter, so it can't be populated any earlier ).
    gl : Option< GL >,
  }

  impl TransformsData
  {
    /// Returns a slice of the resolved joint nodes, in skin-joint-array order.
    #[ must_use ]
    pub fn joints_get( &self ) -> &[ Rc< RefCell< Node > > ]
    {
      self.joints.as_slice()
    }

    /// Creates [`TransformsData`]
    #[ must_use ]
    pub fn new( joints : Vec< ( Rc< RefCell< Node > >, F32x4x4 ) > ) -> Self
    {
      let mut nodes = vec![];
      let mut inverse_bind_matrices = vec![];

      for ( node, matrix ) in joints
      {
        nodes.push( node );
        inverse_bind_matrices.push( matrix );
      }

      Self
      {
        joints : nodes,
        inverse_bind_matrices,
        global_texture : None,
        inverse_texture : None,
        need_update_inverse : true,
        need_clone_inner : false,
        gl : None,
      }
    }

    /// Upload inverse bind matrices texture to current shader program
    fn upload
    (
      &mut self,
      gl : &GL,
      locations : &FxHashMap< String, Option< gl::WebGlUniformLocation > >
    )
    {
      self.gl = Some( gl.clone() );

      if self.need_clone_inner
      {
        self.need_clone_inner =
        gl.create_texture()
        .map( | g | { self.global_texture = Some( g ); } )
        .is_none()
        ||
        gl.create_texture()
        .map( | i | { self.inverse_texture = Some( i ); } )
        .is_none();
      }

      let global_matrices = self.joints.iter()
      .map
      (
        | node | node.borrow().world_matrix_get()
      )
      .collect::< Vec< _ > >();

      let mut global_data = global_matrices.iter()
      .flat_map(| m | m.to_array().to_vec())
      .collect::< Vec< _ > >();

      let a = data_texture_size_calculate(global_data.len() );
      let texture_size = [ a, a ];

      global_data.extend( vec![ 0.0; ( a * a * 4 ) as usize - global_data.len() ] );

      if self.need_update_inverse
      {
        if self.global_texture.is_none()
        {
          self.global_texture = gl.create_texture();
        }
        if self.inverse_texture.is_none()
        {
          self.inverse_texture = gl.create_texture();
        }

        let mut inverse_data = self.inverse_bind_matrices.iter()
        .flat_map(| m | m.to_array().to_vec())
        .collect::< Vec< _ > >();

        inverse_data.extend( vec![ 0.0; ( a * a * 4 ) as usize - inverse_data.len() ] );
        let _ = texture_data_4f_load( gl, self.inverse_texture.as_ref().unwrap(), inverse_data.as_slice(), texture_size );

        if self.inverse_texture.is_some() && self.global_texture.is_some()
        {
          self.need_update_inverse = false;
        }
      }

      if let ( Some( global_texture ), Some( inverse_texture ) ) = ( &self.global_texture, &self.inverse_texture )
      {
        let global_matrices_loc = locations.get( "globalJointTransformMatricesTexture" ).unwrap();
        let inverse_matrices_loc = locations.get( "inverseBindMatricesTexture" ).unwrap();
        let texture_size_loc = locations.get( "skinMatricesTextureSize" ).unwrap();

        let _ = texture_data_4f_load( gl, global_texture, global_data.as_slice(), texture_size );
        texture_upload( gl, global_texture, global_matrices_loc.as_ref(), GLOBAL_MATRICES_SLOT );
        texture_upload( gl, inverse_texture, inverse_matrices_loc.as_ref(), INVERSE_MATRICES_SLOT );
        gl::uniform::upload( gl, texture_size_loc.clone(), texture_size.as_slice() ).unwrap();
      }
    }
  }

  impl Clone for TransformsData
  {
    fn clone( &self ) -> Self
    {
      Self
      {
        joints : self.joints.iter()
        .map( | n | n.borrow().tree_clone() )
        .collect::< Vec< _ > >(),
        inverse_bind_matrices : self.inverse_bind_matrices.clone(),
        global_texture : self.global_texture.clone(),
        inverse_texture : self.inverse_texture.clone(),
        need_update_inverse : true,
        need_clone_inner : true,
        gl : self.gl.clone(),
      }
    }
  }

  // Fix(BUG-437): `TransformsData` allocated `global_texture`/`inverse_texture` via
  // `gl.create_texture()` inside `upload()` but never freed them anywhere -- dropping a
  // `TransformsData` ( e.g. when its owning `Skeleton`/`Mesh`/`Node` is discarded ) silently
  // leaked both GPU textures every time.
  // Root cause: the struct had no `impl Drop` and no manual `gl_resources_free`-style method;
  // nothing in the type ever called `gl.delete_texture` on either field.
  // Pitfall: `Clone` copies `global_texture`/`inverse_texture` by handle ( the same underlying
  // GPU texture, not a deep copy ), relying on `need_clone_inner = true` to force `upload()`
  // to allocate the clone its *own* fresh textures before ever binding/uploading through them
  // ( see the `if self.need_clone_inner { .. }` block in `upload()`, which always runs before
  // any GL call that would actually use the field ). Freeing unconditionally in `Drop` is safe
  // only because of that ordering guarantee -- if a future edit ever read `global_texture`/
  // `inverse_texture` for a GL call *before* the `need_clone_inner` reallocation in `upload()`,
  // dropping the original ahead of the clone's first `upload()` would leave the clone pointing
  // at an already-deleted texture.
  impl Drop for TransformsData
  {
    fn drop( &mut self )
    {
      if let Some( ref gl ) = self.gl
      {
        gl.delete_texture( self.global_texture.as_ref() );
        gl.delete_texture( self.inverse_texture.as_ref() );
      }
    }
  }

  /// Skin morph targets related data
  #[ derive( Debug ) ]
  pub struct DisplacementsData
  {
    /// Morph targets positions displacements
    positions_displacements : Option< Vec< [ f32; 3 ] > >,
    /// Morph targets normals displacements
    normals_displacements : Option< Vec< [ f32; 3 ] > >,
    /// Morph targets tangents displacements
    tangents_displacements : Option< Vec< [ f32; 3 ] > >,
    /// Morph targets displacements texture
    displacements_texture : Option< WebGlTexture >,
    /// [`Self::displacements_texture`] size
    disp_texture_size : [ u32; 2 ],
    /// Morph weights for updating geometry every frame
    morph_weights : Rc< RefCell< Vec< f32 > > >,
    /// Default morph weights
    pub default_weights : Vec< f32 >,
    /// Count of morph targets
    targets_count : usize,
    /// Offsets of each displacement in `One combined vertex multitarget block`
    /// (see docs of [`Self::upload`]). If offset is -1 it's, means that it
    /// doesn't included into [`Self::displacements_texture`] texture
    disp_offsets : I32x3,
    /// Displacements count. Must be sum of mesh primitives vertices count
    vertices_count : usize,
    /// Define if need update [`Self::displacements_texture`]
    need_update_displacement : bool,
    /// Defines if [`DisplacementsData`] is recently cloned,
    /// but not all fields have been cloned too
    need_clone_inner : bool,
    /// GL context `displacements_texture` was allocated from -- retained so `impl Drop` can
    /// free it. `None` until the first `upload()` call actually allocates a texture ( `new()`
    /// receives no `gl` parameter, so it can't be populated any earlier ).
    gl : Option< GL >,
  }

  impl Default for DisplacementsData
  {
    fn default() -> Self
    {
      Self::new()
    }
  }

  impl DisplacementsData
  {
    /// Creates empty [`DisplacementsData`]
    #[ must_use ]
    pub fn new() -> Self
    {
      Self
      {
        positions_displacements : None,
        normals_displacements : None,
        tangents_displacements : None,
        displacements_texture : None,
        disp_texture_size : [ 0; 2 ],
        morph_weights : Rc::new( RefCell::new( vec![] ) ),
        default_weights : vec![],
        targets_count : 0,
        disp_offsets : I32x3::splat( -1 ),
        vertices_count : 0,
        need_update_displacement : false,
        need_clone_inner : false,
        gl : None,
      }
    }

    /// Returns binded attributes count
    #[ must_use ]
    pub fn attributes_count( &self ) -> usize
    {
      [
        &self.positions_displacements,
        &self.normals_displacements,
        &self.tangents_displacements
      ]
      .iter()
      .filter( | v | v.is_some() )
      .count()
    }

    /// Packs displacement data into texture.
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
    /// # Panics
    ///
    /// Does not panic in practice : the `unwrap` runs only on slots pre-filtered to be `Some`.
    pub fn displacements_data_pack( &mut self ) -> Vec< f32 >
    {
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
      .map( std::vec::Vec::len )
      .max()
      .unwrap_or_default();

      let attributes_count = arrays.len();

      self.targets_count = len.checked_div( self.vertices_count ).unwrap_or( 0 );

      let mut data = Vec::with_capacity
      (
        self.vertices_count
        * attributes_count
        * self.targets_count
        * 4
      );

      for v in 0..self.vertices_count
      {
        let vertex_base = v * self.targets_count;

        for arr in &arrays
        {
          for t in 0..self.targets_count
          {
            let d = arr[ vertex_base + t ];
            data.extend_from_slice( &[ d[ 0 ], d[ 1 ], d[ 2 ], 1.0 ] );
          }
        }
      }

      data
    }

    /// Uploads morph targets data to uniforms
    fn upload
    (
      &mut self,
      gl : &GL,
      locations : &FxHashMap< String, Option< gl::WebGlUniformLocation > >
    )
    {
      self.gl = Some( gl.clone() );

      if self.need_clone_inner
      {
        self.need_clone_inner =
        gl.create_texture()
        .map( | d | { self.displacements_texture = Some( d ); } )
        .is_none();
      }

      if self.need_update_displacement && !self.displacements_update( gl )
      {
        return;
      }

      self.uniforms_upload( gl, locations );
    }

    /// Repacks displacement data and reallocates the displacement texture.
    ///
    /// Returns `false` when the required texture would exceed the WebGL size limit — the update is
    /// abandoned and `need_update_displacement` stays set, so the next call retries.
    fn displacements_update( &mut self, gl : &GL ) -> bool
    {
      if self.displacements_texture.is_none()
      {
        self.displacements_texture = gl.create_texture();
      }

      let mut data = self.displacements_data_pack();

      let vertex_displacement_len = self.attributes_count() * self.targets_count;
      if self.morph_weights.borrow().is_empty()
      {
        *self.morph_weights.borrow_mut() = if self.default_weights.len() == self.targets_count
        {
          self.default_weights.clone()
        }
        else
        {
          vec![ 0.0; self.targets_count ]
        };
      }

      if vertex_displacement_len != 0
      {
        let ( a, b ) = displacement_texture_size_compute( data.len(), vertex_displacement_len );

        let max_size = gl.get_parameter( gl::MAX_TEXTURE_SIZE )
        .ok()
        .and_then(| v | v.as_f64())
        .unwrap_or( 0.0 ) as u32;
        if a.max( b ) > max_size
        {
          gl::web::error!
          (
            "Displacement texture size exceeded max WebGL texture size: {:?} > {:?}",
            ( a, b ),
            ( max_size, max_size )
          );
          return false;
        }

        self.disp_texture_size = [ a, b ];
        data.extend( vec![ 0.0; ( a * b * 4 ) as usize - data.len() ] );
        let _ = texture_data_4f_load( gl, self.displacements_texture.as_ref().unwrap(), data.as_slice(), [ a, b ] );
      }

      let mut offset = 0_i32;
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

      self.disp_offsets = I32x3::from_array( offsets );

      self.need_update_displacement = false;
      true
    }

    /// Uploads the displacement texture and morph-target uniforms to their locations.
    fn uniforms_upload
    (
      &self,
      gl : &GL,
      locations : &FxHashMap< String, Option< gl::WebGlUniformLocation > >
    )
    {
      if let Some( displacements_texture ) = &self.displacements_texture
      {
        if let Some( displacements_loc ) = locations.get( "morphTargetsDisplacementsTexture" )
        {
          texture_upload( gl, displacements_texture, displacements_loc.as_ref(), DISPLACEMENTS_SLOT );
        }
        if let Some( morph_weights_loc ) = locations.get( "morphWeights" )
        {
          let mut data = self.morph_weights
          .borrow()
          .get( 0..self.targets_count )
          .map( | v | v.iter().map( | i | [ *i; 1 ] ).collect::< Vec< _ > >() )
          .unwrap_or( vec![ [ 0.0_f32; 1 ]; self.targets_count ] );
          data.extend( vec![ [ 0.0; 1 ]; MAX_MORPH_TARGETS.saturating_sub( data.len() ) ] );
          gl::uniform::upload
          (
            gl,
            morph_weights_loc.clone(),
            data.as_slice()
          )
          .unwrap();
        }
        if let Some( disp_size_loc ) = locations.get( "displacementsTextureSize" )
        {
          gl::uniform::upload( gl, disp_size_loc.clone(), self.disp_texture_size.as_slice() ).unwrap();
        }
        if let Some( targets_count_loc ) = locations.get( "morphTargetsCount" )
        {
          gl::uniform::upload( gl, targets_count_loc.clone(), &( self.targets_count as u32 ) ).unwrap();
        }
        if let Some( disp_offsets_loc ) = locations.get( "morphTargetsDisplacementsOffsets" )
        {
          gl::uniform::upload( gl, disp_offsets_loc.clone(), &self.disp_offsets.to_array()[ .. ] ).unwrap();
        }
      }
    }

    /// Returns morph weights that is used for updating geometry
    #[ must_use ]
    pub fn morph_weights_get( &self ) -> Rc< RefCell< Vec< f32 > > >
    {
      self.morph_weights.clone()
    }

    /// Sets one morph targets vertex attribute data that will be packed into texture
    pub fn displacement_set
    (
      &mut self,
      displacement_array : Option< Vec< [ f32; 3 ] > >,
      displacement_type : &gltf::Semantic,
      vertices_count : usize
    )
    -> bool
    {
      if vertices_count != self.vertices_count && self.vertices_count > 0 && displacement_array.is_some()
      {
        return false;
      }

      if displacement_array.is_some()
      {
        self.vertices_count = vertices_count;
      }

      let positions_len = self.positions_displacements.as_ref().map( std::vec::Vec::len ).unwrap_or_default();
      let normals_len = self.normals_displacements.as_ref().map( std::vec::Vec::len ).unwrap_or_default();
      let tangents_len = self.tangents_displacements.as_ref().map( std::vec::Vec::len ).unwrap_or_default();
      let mut unique =
      [
        displacement_array.as_ref().map_or( 0, std::vec::Vec::len ),
        positions_len,
        normals_len,
        tangents_len
      ]
      .into_iter()
      .collect::< FxHashSet< _ > >();
      unique.remove( &0 );
      if unique.len() > 1
      {
        return false;
      }

      let displacement_is_some = displacement_array.is_some();

      match displacement_type
      {
        gltf::Semantic::Positions => { self.positions_displacements = displacement_array; },
        gltf::Semantic::Normals => { self.normals_displacements = displacement_array; },
        gltf::Semantic::Tangents => { self.tangents_displacements = displacement_array; }
        _ => return false
      }

      if self.displacements_texture.is_some() || displacement_is_some
      {
        match displacement_type
        {
          gltf::Semantic::Positions |
          gltf::Semantic::Normals |
          gltf::Semantic::Tangents => { self.need_update_displacement = true; }
          _ => return false
        }
      }

      true
    }
  }

  impl Clone for DisplacementsData
  {
    fn clone( &self ) -> Self
    {
      Self
      {
        positions_displacements : self.positions_displacements.clone(),
        normals_displacements : self.normals_displacements.clone(),
        tangents_displacements : self.tangents_displacements.clone(),
        displacements_texture : self.displacements_texture.clone(),
        disp_texture_size : self.disp_texture_size,
        morph_weights : Rc::new( RefCell::new( self.morph_weights.borrow().clone() ) ),
        default_weights : self.default_weights.clone(),
        targets_count : self.targets_count,
        disp_offsets : self.disp_offsets,
        vertices_count : self.vertices_count,
        need_update_displacement : true,
        need_clone_inner : true,
        gl : self.gl.clone(),
      }
    }
  }

  // Fix(BUG-437): `DisplacementsData` allocated `displacements_texture` via
  // `gl.create_texture()` inside `upload()` but never freed it anywhere -- dropping a
  // `DisplacementsData` ( e.g. when its owning `Skeleton`/`Mesh`/`Node` is discarded ) silently
  // leaked the GPU texture every time.
  // Root cause: the struct had no `impl Drop` and no manual `gl_resources_free`-style method;
  // nothing in the type ever called `gl.delete_texture` on the field.
  // Pitfall: `Clone` copies `displacements_texture` by handle ( the same underlying GPU
  // texture, not a deep copy ), relying on `need_clone_inner = true` to force `upload()` to
  // allocate the clone its *own* fresh texture before ever binding/uploading through it ( see
  // the `if self.need_clone_inner { .. }` block in `upload()`, which always runs before
  // `displacements_update()` would otherwise reuse an existing `Some` handle ). Freeing
  // unconditionally in `Drop` is safe only because of that ordering guarantee -- see the
  // identical caveat on `TransformsData`'s `impl Drop` above.
  impl Drop for DisplacementsData
  {
    fn drop( &mut self )
    {
      if let Some( ref gl ) = self.gl
      {
        gl.delete_texture( self.displacements_texture.as_ref() );
      }
    }
  }

  /// Set of virtual bones used to deform and control the
  /// movement of a 3D models. It's a fundamental concept
  /// in skeletal animation, the most common method for
  /// rigging and animating complex models.
  ///
  /// This implementation conserns that skeleton is combination
  /// of joints transform data and morph targets dispalcements
  /// data
  #[ derive( Debug, Clone ) ]
  pub struct Skeleton
  {
    /// Data related to joint transforms
    transforms : Option< TransformsData >,
    /// Data related to morph targets
    displacements : Option< DisplacementsData >
  }

  impl Default for Skeleton
  {
    fn default() -> Self
    {
      Self::new()
    }
  }

  impl Skeleton
  {
    /// Creates a new [`Skeleton`] instance
    #[ must_use ]
    pub fn new() -> Self
    {
      Self
      {
        transforms : None,
        displacements : None
      }
    }

    /// Upload joints transform and morph targets displacements data
    pub fn upload
    (
      &mut self,
      gl : &GL,
      locations : &FxHashMap< String, Option< gl::WebGlUniformLocation > >
    )
    {
      if let Some( t ) = self.transforms.as_mut()
      {
        t.upload( gl, locations );
      }
      if let Some( d ) = self.displacements.as_mut()
      {
        d.upload( gl, locations );
      }
    }

    /// Get `Self::transforms` as reference
    #[ must_use ]
    pub fn transforms_as_ref( &self ) -> &Option< TransformsData >
    {
      &self.transforms
    }

    /// Get `Self::transforms` as mutable reference
    pub fn transforms_as_mut( &mut self ) -> &mut Option< TransformsData >
    {
      &mut self.transforms
    }

    /// Get `Self::displacements` as reference
    #[ must_use ]
    pub fn displacements_as_ref( &self ) -> &Option< DisplacementsData >
    {
      &self.displacements
    }

    /// Get `Self::displacements` as mutable reference
    pub fn displacements_as_mut( &mut self ) -> &mut Option< DisplacementsData >
    {
      &mut self.displacements
    }

    /// Can be used for checking if skin is available at this [`Skeleton`]
    #[ must_use ]
    pub fn has_skin( &self ) -> bool
    {
      self.transforms.is_some()
    }

    /// Can be used for checking if morph targets are available at this [`Skeleton`]
    #[ must_use ]
    pub fn has_morph_targets( &self ) -> bool
    {
      self.displacements.is_some()
    }
  }

  // Test placement: both tests construct `TransformsData`/`DisplacementsData` directly via a
  // struct literal ( bypassing the real `new()`/`upload()` call chain, which needs real `Node`
  // instances and real shader uniform locations neither test cares about ) and read the private
  // `global_texture`/`inverse_texture`/`displacements_texture` fields before drop -- both only
  // possible from a test nested inside `mod private`. See `rulebook.md § Test placement`.
  #[ cfg( all( test, target_arch = "wasm32" ) ) ]
  mod tests
  {
    use super::*;

    fn gl_init() -> GL
    {
      gl::browser::setup( gl::browser::Config::default() );
      let options = gl::context::ContextOptions::default();
      let canvas = gl::canvas::make().unwrap();
      gl::context::from_canvas_with( &canvas, options ).unwrap()
    }

    /// ## Root Cause
    /// `TransformsData` allocated `global_texture`/`inverse_texture` via `gl.create_texture()`
    /// inside `upload()` but never freed them anywhere -- dropping a `TransformsData` ( e.g.
    /// when its owning `Skeleton`/`Mesh`/`Node` is discarded ) silently leaked both textures.
    ///
    /// ## Why Not Caught
    /// `skeleton_tests.rs` and `gltf_skeleton_displacements_test.rs` exercise upload/animation
    /// logic but never construct-then-drop a `TransformsData` to check for leaked GL objects.
    ///
    /// ## Fix Applied
    /// Added a `gl : Option< GL >` field ( populated on first `upload()` call ) and
    /// `impl Drop for TransformsData`, deleting `global_texture`/`inverse_texture` when `gl` is
    /// populated.
    ///
    /// ## Prevention
    /// Constructs a `TransformsData` directly via struct literal with `global_texture`/
    /// `inverse_texture` pre-populated and `gl` set ( bypassing `upload()`'s real allocation
    /// path, which is exercised separately by `skeleton_tests.rs` ), then asserts both handles
    /// are freed after drop -- the same deterministic existence-check pattern used by this
    /// crate's other GPU-teardown reproducer tests.
    ///
    /// ## Pitfall
    /// A struct whose GPU-resource-owning fields are only populated lazily ( on first `upload`,
    /// not in `new` ) is easy to reason about as "doesn't own anything yet" and skip when
    /// auditing for missing `Drop` impls -- the fields are still owned once populated, on
    /// whichever call path first fills them in.
    // test_kind: bug_reproducer(BUG-437)
    #[ wasm_bindgen_test::wasm_bindgen_test ]
    fn transforms_data_drop_frees_global_and_inverse_textures()
    {
      let gl = gl_init();
      let global_texture = gl.create_texture();
      let inverse_texture = gl.create_texture();
      // Test pitfall (not a production bug): `create_texture()` alone allocates a name, but
      // `isTexture` only recognizes it once bound at least once via `bindTexture` -- every real
      // `upload()` call binds before use, so this one-time bind reproduces that precondition.
      gl.bind_texture( gl::TEXTURE_2D, global_texture.as_ref() );
      gl.bind_texture( gl::TEXTURE_2D, inverse_texture.as_ref() );
      gl.bind_texture( gl::TEXTURE_2D, None );
      assert!( gl.is_texture( global_texture.as_ref() ) );
      assert!( gl.is_texture( inverse_texture.as_ref() ) );

      let transforms_data = TransformsData
      {
        joints : vec![],
        inverse_bind_matrices : vec![],
        global_texture : global_texture.clone(),
        inverse_texture : inverse_texture.clone(),
        need_update_inverse : false,
        need_clone_inner : false,
        gl : Some( gl.clone() ),
      };

      drop( transforms_data );

      assert!( !gl.is_texture( global_texture.as_ref() ), "TransformsData::drop must delete global_texture" );
      assert!( !gl.is_texture( inverse_texture.as_ref() ), "TransformsData::drop must delete inverse_texture" );
    }

    /// ## Root Cause
    /// `DisplacementsData` allocated `displacements_texture` via `gl.create_texture()` inside
    /// `upload()` but never freed it anywhere -- dropping a `DisplacementsData` silently leaked
    /// the GPU texture every time.
    ///
    /// ## Why Not Caught
    /// Same gap as `TransformsData` above -- no test previously constructed-then-dropped a
    /// `DisplacementsData` to check for a leaked GL object.
    ///
    /// ## Fix Applied
    /// Added a `gl : Option< GL >` field ( populated on first `upload()` call ) and
    /// `impl Drop for DisplacementsData`, deleting `displacements_texture` when `gl` is
    /// populated.
    ///
    /// ## Prevention
    /// Constructs a `DisplacementsData` directly via struct literal with `displacements_texture`
    /// pre-populated and `gl` set, then asserts the handle is freed after drop.
    ///
    /// ## Pitfall
    /// Same as `TransformsData` above -- lazily-populated GPU fields are still owned once
    /// populated, regardless of which call path first fills them in.
    // test_kind: bug_reproducer(BUG-437)
    #[ wasm_bindgen_test::wasm_bindgen_test ]
    fn displacements_data_drop_frees_displacements_texture()
    {
      let gl = gl_init();
      let displacements_texture = gl.create_texture();
      // Test pitfall (not a production bug): see the identical comment on
      // `transforms_data_drop_frees_global_and_inverse_textures` above.
      gl.bind_texture( gl::TEXTURE_2D, displacements_texture.as_ref() );
      gl.bind_texture( gl::TEXTURE_2D, None );
      assert!( gl.is_texture( displacements_texture.as_ref() ) );

      let displacements_data = DisplacementsData
      {
        positions_displacements : None,
        normals_displacements : None,
        tangents_displacements : None,
        displacements_texture : displacements_texture.clone(),
        disp_texture_size : [ 0; 2 ],
        morph_weights : Rc::new( RefCell::new( vec![] ) ),
        default_weights : vec![],
        targets_count : 0,
        disp_offsets : I32x3::splat( -1 ),
        vertices_count : 0,
        need_update_displacement : false,
        need_clone_inner : false,
        gl : Some( gl.clone() ),
      };

      drop( displacements_data );

      assert!( !gl.is_texture( displacements_texture.as_ref() ), "DisplacementsData::drop must delete displacements_texture" );
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    texture_data_4f_load,
    data_texture_size_calculate,
    displacement_texture_size_compute,
    TransformsData,
    DisplacementsData,
    Skeleton,
    GLOBAL_MATRICES_SLOT,
    INVERSE_MATRICES_SLOT,
    DISPLACEMENTS_SLOT,
    MAX_MORPH_TARGETS
  };
}
