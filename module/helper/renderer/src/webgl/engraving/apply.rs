mod private
{
  use std::{ cell::RefCell, rc::Rc };
  use minwebgl as gl;
  use gl::GL;
  use rustc_hash::FxHashMap;
  use crate::webgl::
  {
    Material,
    Mesh,
    Node,
    Object3D,
    Primitive,
    Scene,
    material::PbrMaterial,
    engraving::{ EngravingBitmap, EngravingCanvasError, EngravingCanvasManager, EngravingConfig, EngravingTextLayout, EngravingTexture },
  };

  /// Error produced while binding an [`EngravingConfig`] to a loaded [`Scene`], or while
  /// re-rendering text for one of its nodes.
  #[ derive( Debug ) ]
  pub enum EngravingError
  {
    /// No node named this way exists in the scene.
    NodeNotFound( Box< str > ),
    /// The node exists but isn't a mesh (or its mesh has no primitives), so there is nothing to
    /// attach an engraving texture to.
    NotAMesh( Box< str > ),
    /// The node's primitive material isn't a [`PbrMaterial`] — engraving is currently only
    /// wired into the PBR material path (see `main.frag`'s `USE_ENGRAVING` block).
    MaterialNotPbr( Box< str > ),
    /// This session has no entry for the requested node name (it wasn't in the config passed to
    /// [`EngravingSession::build`]).
    UnknownNode( Box< str > ),
    /// [`EngravingSession::set_text`] was asked for a font that isn't in the node's
    /// `allowedFonts` whitelist.
    FontNotAllowed
    {
      /// The node the text was for.
      node : Box< str >,
      /// The rejected CSS font family.
      font : Box< str >,
    },
    /// A WebGL call failed (texture allocation/update).
    Gl( gl::WebglError ),
    /// Font loading or text rasterization failed.
    Canvas( EngravingCanvasError ),
  }

  impl std::fmt::Display for EngravingError
  {
    fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
    {
      match self
      {
        Self::NodeNotFound( n ) => write!( f, "no node named '{n}' in the scene" ),
        Self::NotAMesh( n ) => write!( f, "node '{n}' is not a mesh with at least one primitive" ),
        Self::MaterialNotPbr( n ) => write!( f, "node '{n}'s material is not a PbrMaterial" ),
        Self::UnknownNode( n ) => write!( f, "no engraving config for node '{n}'" ),
        Self::FontNotAllowed { node, font } => write!( f, "font '{font}' is not in node '{node}'s allowedFonts" ),
        Self::Gl( e ) => write!( f, "{e}" ),
        Self::Canvas( e ) => write!( f, "{e}" ),
      }
    }
  }

  impl std::error::Error for EngravingError {}

  impl From< gl::WebglError > for EngravingError
  {
    fn from( e : gl::WebglError ) -> Self { Self::Gl( e ) }
  }

  impl From< EngravingCanvasError > for EngravingError
  {
    fn from( e : EngravingCanvasError ) -> Self { Self::Canvas( e ) }
  }

  struct EngravingEntry
  {
    /// Only the mutable-safe subset of the node's config — see [`EngravingTextLayout`] for why
    /// the geometry fields (`uv_channel`, `aspect_ratio`, `texture_height`, ...) aren't here at
    /// all, rather than merely being documented as unsafe to touch.
    layout : EngravingTextLayout,
    /// The node's `allowedFonts`, checked by [`EngravingSession::set_text`].
    allowed_fonts : Vec< Box< str > >,
    canvas : EngravingCanvasManager,
    /// Re-rendering text only needs to update the GPU texture in place ([`EngravingTexture::update`]);
    /// every `PbrMaterial` this was attached to already shares it via `Rc<RefCell<Texture>>`
    /// (see [`Self::build`]), so no reference back to those materials needs to be kept here.
    texture : EngravingTexture,
  }

  /// Binds one [`EngravingConfig`] to a loaded [`Scene`]: for every configured node it allocates
  /// a persistent offscreen canvas and GPU mask texture, and wires the texture (plus the node's
  /// bevel/roughness/darkening knobs) into that node's own copy of its [`PbrMaterial`]s. Call
  /// [`Self::set_text`] afterwards, as many times as needed, to actually rasterize and upload
  /// engraving text.
  pub struct EngravingSession
  {
    entries : FxHashMap< Box< str >, EngravingEntry >,
  }

  /// A material slot as stored on a [`Primitive`].
  type SharedMaterial = Rc< RefCell< Box< dyn Material > > >;

  /// One configured node, resolved against the scene and checked before anything is mutated.
  struct ResolvedNode
  {
    node : Rc< RefCell< Node > >,
    mesh : Rc< RefCell< Mesh > >,
  }

  /// Resolves `node_name` to a mesh node of `scene` whose every primitive has a [`PbrMaterial`].
  fn node_resolve( scene : &Scene, node_name : &str ) -> Result< ResolvedNode, EngravingError >
  {
    let node = scene.node_get( node_name )
    .ok_or_else( || EngravingError::NodeNotFound( node_name.into() ) )?;

    let mesh =
    {
      let node_ref = node.borrow();
      let Object3D::Mesh( mesh ) = &node_ref.object
      else
      {
        return Err( EngravingError::NotAMesh( node_name.into() ) );
      };
      mesh.clone()
    };

    {
      let mesh_ref = mesh.borrow();
      if mesh_ref.primitives.is_empty()
      {
        return Err( EngravingError::NotAMesh( node_name.into() ) );
      }
      for primitive in &mesh_ref.primitives
      {
        if primitive.borrow().material.borrow().as_any().downcast_ref::< PbrMaterial >().is_none()
        {
          return Err( EngravingError::MaterialNotPbr( node_name.into() ) );
        }
      }
    }

    Ok( ResolvedNode { node, mesh } )
  }

  /// Number of nodes of `scene` whose object is exactly `mesh` (the same `Rc`).
  fn mesh_user_count( scene : &Scene, mesh : &Rc< RefCell< Mesh > > ) -> usize
  {
    let mut count = 0;
    let _ = scene.traverse
    (
      &mut | node : Rc< RefCell< Node > > |
      {
        if let Object3D::Mesh( other ) = &node.borrow().object
        {
          if Rc::ptr_eq( other, mesh )
          {
            count += 1;
          }
        }
        Ok( () )
      }
    );
    count
  }

  /// Gives `resolved.node` its own copy of every material its primitives use, swaps the copies
  /// into the primitives, and returns them.
  ///
  /// The glTF loader hands out one material per (glTF material, vertex defines) pair to every
  /// primitive that uses it, across all meshes and nodes. Writing the engraving texture into that
  /// shared material would engrave every other node using it too, and two configured nodes
  /// sharing it would fight over one texture slot. Primitives of this node that shared one
  /// material keep sharing one copy.
  ///
  /// glTF mesh instancing puts one [`Mesh`], and so the same primitives, on several nodes. When
  /// another node of `scene` uses this node's mesh, the node first gets its own [`Mesh`] with new
  /// primitives over the same geometry and skeleton, so swapping a primitive's material can't
  /// reach the other node.
  fn node_materials_isolate( scene : &Scene, resolved : &ResolvedNode ) -> Vec< SharedMaterial >
  {
    let mesh = if mesh_user_count( scene, &resolved.mesh ) > 1
    {
      let detached =
      {
        let mesh_ref = resolved.mesh.borrow();
        Mesh
        {
          primitives : mesh_ref.primitives.iter()
          .map
          (
            | p |
            {
              let p = p.borrow();
              Rc::new( RefCell::new( Primitive { geometry : p.geometry.clone(), material : p.material.clone() } ) )
            }
          )
          .collect(),
          skeleton : mesh_ref.skeleton.clone(),
          is_shadow_caster : mesh_ref.is_shadow_caster,
        }
      };
      let detached = Rc::new( RefCell::new( detached ) );
      resolved.node.borrow_mut().object = Object3D::Mesh( detached.clone() );
      detached
    }
    else
    {
      resolved.mesh.clone()
    };

    let mut copies : Vec< ( SharedMaterial, SharedMaterial ) > = Vec::new();
    for primitive in &mesh.borrow().primitives
    {
      let original = primitive.borrow().material.clone();
      let copy = if let Some( ( _, copy ) ) = copies.iter().find( | ( o, _ ) | Rc::ptr_eq( o, &original ) )
      {
        copy.clone()
      }
      else
      {
        let copy : SharedMaterial = Rc::new( RefCell::new( original.borrow().dyn_clone() ) );
        copies.push( ( original, copy.clone() ) );
        copy
      };
      primitive.borrow_mut().material = copy;
    }

    copies.into_iter().map( | ( _, copy ) | copy ).collect()
  }

  impl EngravingSession
  {
    /// Walks `config.nodes`, resolving each `nodeName` against `scene` and attaching a fresh
    /// [`EngravingTexture`] to that node's `PbrMaterial`s.
    ///
    /// Each configured node gets its own copy of every material on its primitives
    /// (copy-on-write), swapped into the primitives before the texture is set. The glTF loader
    /// shares one material between every primitive using the same glTF material, so engraving
    /// it in place would engrave every other node using it too. A material handle read from the
    /// node *before* this call is therefore no longer the one the node renders with; read it
    /// from the node's primitives again afterwards. When another node of `scene` uses the same
    /// [`Mesh`] (glTF mesh instancing), the configured node first gets its own `Mesh` over the
    /// same geometry and skeleton.
    ///
    /// Every node is resolved and checked before anything is allocated or mutated, so a
    /// resolution error leaves `scene` untouched. Partial application would leave some
    /// materials silently un-engravable, which is worth surfacing rather than skipping quietly.
    ///
    /// # Errors
    ///
    /// [`EngravingError::NodeNotFound`], [`EngravingError::NotAMesh`] or
    /// [`EngravingError::MaterialNotPbr`] for the first node that can't take an engraving, and
    /// [`EngravingError::Canvas`] / [`EngravingError::Gl`] if a node's canvas or texture can't be
    /// created.
    ///
    /// # Panics
    ///
    /// Panics if a node, mesh, primitive or material of `scene` is already mutably borrowed
    /// while this runs. The `PbrMaterial` downcast can't fail: every material is checked first.
    pub fn build( gl : &GL, scene : &Scene, config : &EngravingConfig ) -> Result< Self, EngravingError >
    {
      let resolved = config.nodes.iter()
      .map( | node_config | node_resolve( scene, &node_config.node_name ) )
      .collect::< Result< Vec< _ >, _ > >()?;

      let mut allocated = Vec::with_capacity( config.nodes.len() );
      for node_config in &config.nodes
      {
        let ( width, height ) = node_config.texture_size();
        let canvas = EngravingCanvasManager::new( width, height )?;
        let texture = EngravingTexture::new( gl, width, height, node_config.uv_channel )?;
        allocated.push( ( canvas, texture ) );
      }

      let mut entries = FxHashMap::default();
      for ( ( node_config, resolved ), ( canvas, texture ) ) in config.nodes.iter().zip( &resolved ).zip( allocated )
      {
        for material in node_materials_isolate( scene, resolved )
        {
          let mut material_mut = material.borrow_mut();
          let pbr = material_mut.as_any_mut().downcast_mut::< PbrMaterial >()
          .expect( "checked by node_resolve" );

          pbr.engraving_texture_set( Some( texture.texture_info() ) );
          pbr.engraving_strength = node_config.engraving_strength;
          pbr.engraving_roughness = node_config.engraving_roughness;
          pbr.engraving_darkening = node_config.engraving_darkening;
        }

        entries.insert( node_config.node_name.clone(), EngravingEntry
        {
          layout : node_config.text_layout(),
          allowed_fonts : node_config.allowed_fonts.clone(),
          canvas,
          texture,
        } );
      }

      Ok( Self { entries } )
    }

    /// Renders `text` (in `font_family`, which must be one of the node's `allowed_fonts`) for
    /// the node named `node_name` and uploads its full mip chain via `texSubImage2D` per level —
    /// no reallocation, no shader recompilation (the `PbrMaterial`'s `USE_ENGRAVING` define was
    /// already turned on by [`Self::build`], for every node in the config, whether or not it has
    /// text yet).
    ///
    /// # Errors
    ///
    /// [`EngravingError::UnknownNode`] if the session has no entry for `node_name`,
    /// [`EngravingError::FontNotAllowed`] if `font_family` isn't in that node's `allowedFonts`,
    /// [`EngravingError::Canvas`] if rasterizing fails (see
    /// [`EngravingCanvasManager::render_text_mip_chain`]), or [`EngravingError::Gl`] if the
    /// upload fails (see [`EngravingTexture::update`]).
    pub async fn set_text( &mut self, gl : &GL, node_name : &str, text : &str, font_family : &str ) -> Result< (), EngravingError >
    {
      let entry = self.entries.get_mut( node_name )
      .ok_or_else( || EngravingError::UnknownNode( node_name.into() ) )?;

      if !entry.allowed_fonts.iter().any( | f | **f == *font_family )
      {
        return Err( EngravingError::FontNotAllowed { node : node_name.into(), font : font_family.into() } );
      }

      let bitmaps : Vec< EngravingBitmap > = entry.canvas.render_text_mip_chain
      (
        text,
        font_family,
        &entry.layout,
      ).await?;

      entry.texture.update( gl, &bitmaps )?;

      Ok( () )
    }

    /// Read-only access to a node's current text layout, e.g. to seed GUI controls.
    #[ must_use ]
    pub fn text_layout( &self, node_name : &str ) -> Option< &EngravingTextLayout >
    {
      self.entries.get( node_name ).map( | e | &e.layout )
    }

    /// Mutable access to a node's text layout, for live-tweaking it (`padding`, `align`,
    /// `sizing_mode`, `strip_height_mm`, `default_font_size_mm`, `min_font_size_mm`,
    /// `font_size_ratio`) between [`Self::set_text`] calls — e.g. from a GUI slider. Call
    /// [`Self::set_text`] again afterwards (with the same text) to re-render with the change;
    /// mutating this alone does not touch the GPU texture.
    ///
    /// Unlike a raw `&mut EngravingNodeConfig` would be, this can't desync the canvas/GPU
    /// texture that [`Self::build`] already allocated: [`EngravingTextLayout`] simply has no
    /// field for `node_name`/`uv_channel`/`aspect_ratio`/`texture_height`, so there's nothing
    /// here capable of invalidating that sizing.
    pub fn text_layout_mut( &mut self, node_name : &str ) -> Option< &mut EngravingTextLayout >
    {
      self.entries.get_mut( node_name ).map( | e | &mut e.layout )
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    EngravingSession,
    EngravingError,
  };
}
