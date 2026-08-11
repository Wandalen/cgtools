mod private
{
  use minwebgl as gl;
  use gl::GL;
  use rustc_hash::FxHashMap;
  use crate::webgl::
  {
    Object3D,
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
    canvas : EngravingCanvasManager,
    /// Re-rendering text only needs to update the GPU texture in place ([`EngravingTexture::update`]);
    /// every `PbrMaterial` this was attached to already shares it via `Rc<RefCell<Texture>>`
    /// (see [`Self::build`]), so no reference back to those materials needs to be kept here.
    texture : EngravingTexture,
  }

  /// Binds one [`EngravingConfig`] to a loaded [`Scene`]: for every configured node it allocates
  /// a persistent offscreen canvas and GPU mask texture, and wires the texture (plus the node's
  /// bevel/roughness/darkening knobs) into that node's [`PbrMaterial`]. Call [`Self::set_text`]
  /// afterwards, as many times as needed (e.g. once per keystroke), to actually rasterize and
  /// upload engraving text.
  pub struct EngravingSession
  {
    entries : FxHashMap< Box< str >, EngravingEntry >,
  }

  impl EngravingSession
  {
    /// Walks `config.nodes`, resolving each `nodeName` against `scene` and attaching a fresh
    /// [`EngravingTexture`] to every `PbrMaterial` found on that node's primitives.
    ///
    /// Fails on the first node that can't be resolved (missing node, non-mesh node, or a
    /// material that isn't `PbrMaterial`) — partial application would leave some materials
    /// silently un-engravable, which is worth surfacing rather than skipping quietly.
    pub fn build( gl : &GL, scene : &Scene, config : &EngravingConfig ) -> Result< Self, EngravingError >
    {
      let mut entries = FxHashMap::default();

      for node_config in &config.nodes
      {
        let node = scene.get_node( &node_config.node_name )
        .ok_or_else( || EngravingError::NodeNotFound( node_config.node_name.clone() ) )?;

        let materials : Vec< _ > =
        {
          let node_ref = node.borrow();
          let Object3D::Mesh( mesh ) = &node_ref.object
          else
          {
            return Err( EngravingError::NotAMesh( node_config.node_name.clone() ) );
          };

          let mesh_ref = mesh.borrow();
          if mesh_ref.primitives.is_empty()
          {
            return Err( EngravingError::NotAMesh( node_config.node_name.clone() ) );
          }

          mesh_ref.primitives.iter().map( | p | p.borrow().material.clone() ).collect()
        };

        for material in &materials
        {
          if material.borrow().as_any().downcast_ref::< PbrMaterial >().is_none()
          {
            return Err( EngravingError::MaterialNotPbr( node_config.node_name.clone() ) );
          }
        }

        let ( width, height ) = node_config.texture_size();
        let canvas = EngravingCanvasManager::new( width, height )?;
        let texture = EngravingTexture::new( gl, width, height, node_config.uv_channel )?;

        for material in &materials
        {
          let mut material_mut = material.borrow_mut();
          let pbr = material_mut.as_any_mut().downcast_mut::< PbrMaterial >()
          .expect( "checked above" );

          pbr.set_engraving_texture( Some( texture.texture_info() ) );
          pbr.engraving_strength = node_config.engraving_strength;
          pbr.engraving_roughness = node_config.engraving_roughness;
          pbr.engraving_darkening = node_config.engraving_darkening;
        }

        entries.insert( node_config.node_name.clone(), EngravingEntry { layout : node_config.text_layout(), canvas, texture } );
      }

      Ok( Self { entries } )
    }

    /// Renders `text` (in `font_family`, which must be one of the node's `allowed_fonts`) for
    /// the node named `node_name` and uploads its full mip chain via `texSubImage2D` per level —
    /// no reallocation, no shader recompilation (the `PbrMaterial`'s `USE_ENGRAVING` define was
    /// already turned on by [`Self::build`], for every node in the config, whether or not it has
    /// text yet).
    pub async fn set_text( &mut self, gl : &GL, node_name : &str, text : &str, font_family : &str ) -> Result< (), EngravingError >
    {
      let entry = self.entries.get_mut( node_name )
      .ok_or_else( || EngravingError::UnknownNode( node_name.into() ) )?;

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
