mod private
{
  use serde::Deserialize;

  /// Error returned when an `engraving_config.json` document fails to parse or fails validation.
  ///
  /// Kept as an owned-message struct (instead of a variant on the crate-wide [`crate::webgl::WebglError`])
  /// since parse failures need to carry the underlying `serde_json` diagnostic, which is not `'static`.
  #[ derive( Debug ) ]
  pub struct EngravingConfigError( String );

  impl std::fmt::Display for EngravingConfigError
  {
    fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
    {
      write!( f, "invalid engraving config: {}", self.0 )
    }
  }

  impl std::error::Error for EngravingConfigError {}

  /// Root of `engraving_config.json` — the full set of engravable nodes for one glTF asset.
  ///
  /// See `engraving_config.schema.json` (JSON Schema) and `engraving_config.example.json`
  /// (a worked example) next to this file for the on-disk format.
  #[ derive( Debug, Clone, Deserialize ) ]
  #[ serde( deny_unknown_fields ) ]
  pub struct EngravingConfig
  {
    /// One entry per engravable node. A node not listed here has no engraving support.
    pub nodes : Vec< EngravingNodeConfig >,
  }

  impl EngravingConfig
  {
    /// Parses an `engraving_config.json` document from its raw text.
    pub fn from_json( json : &str ) -> Result< Self, EngravingConfigError >
    {
      let config : Self = serde_json::from_str( json ).map_err( | e | EngravingConfigError( e.to_string() ) )?;

      for node in &config.nodes
      {
        node.validate()?;
      }

      Ok( config )
    }

    /// Returns the config entry for `node_name`, if one is present.
    pub fn node( &self, node_name : &str ) -> Option< &EngravingNodeConfig >
    {
      self.nodes.iter().find( | n | &*n.node_name == node_name )
    }
  }

  /// Engraving settings for a single glTF mesh node.
  #[ derive( Debug, Clone, Deserialize ) ]
  #[ serde( deny_unknown_fields ) ]
  pub struct EngravingNodeConfig
  {
    /// Exact name of the mesh node in the glTF file.
    #[ serde( rename = "nodeName" ) ]
    pub node_name : Box< str >,
    /// Index of the UV set dedicated to engraving (0-4, matching `vUv_0`..`vUv_4` in the shader).
    #[ serde( rename = "uvChannel" ) ]
    pub uv_channel : u32,
    /// `width / height` of the engraving zone, e.g. `4.0` for a 1024x256 strip.
    #[ serde( rename = "aspectRatio" ) ]
    pub aspect_ratio : f32,
    /// Upper bound on text length, enforced before rasterizing.
    #[ serde( rename = "maxCharacters" ) ]
    pub max_characters : usize,
    /// CSS font-family used when the caller doesn't pick one explicitly. Must also appear in `allowed_fonts`.
    #[ serde( rename = "defaultFont" ) ]
    pub default_font : Box< str >,
    /// Whitelist of CSS font-family names selectable for this node.
    #[ serde( rename = "allowedFonts" ) ]
    pub allowed_fonts : Vec< Box< str > >,
    /// Height in pixels of the offscreen canvas / GPU texture. Width is `round( texture_height * aspect_ratio )`.
    #[ serde( rename = "textureHeight", default = "default_texture_height" ) ]
    pub texture_height : u32,
    /// Fractional padding (of the shorter canvas dimension) kept empty around the text on every side.
    #[ serde( default = "default_padding" ) ]
    pub padding : f32,
    /// Intensity of the derivative-based normal perturbation at letter edges (bevel depth).
    #[ serde( rename = "engravingStrength", default = "default_engraving_strength" ) ]
    pub engraving_strength : f32,
    /// Target PBR roughness inside the carved groove.
    #[ serde( rename = "engravingRoughness", default = "default_engraving_roughness" ) ]
    pub engraving_roughness : f32,
    /// Fraction by which the base albedo / specular color is darkened inside the groove.
    #[ serde( rename = "engravingDarkening", default = "default_engraving_darkening" ) ]
    pub engraving_darkening : f32,
    /// Explicit text-sizing strategy. Optional; if omitted, resolves to `HYBRID` when both
    /// `stripHeightMm` and `defaultFontSizeMm` are supplied, otherwise `RELATIVE` — see
    /// [`EngravingNodeConfig::resolved_sizing_mode`].
    #[ serde( rename = "sizingMode", default ) ]
    pub sizing_mode : Option< SizingMode >,
    /// Physical height (mm) of the `TEXCOORD_1` engraving strip on the real object. Required by
    /// `PHYSICAL`/`HYBRID` sizing (whether resolved implicitly or set explicitly).
    #[ serde( rename = "stripHeightMm", default ) ]
    pub strip_height_mm : Option< f32 >,
    /// Target physical font height (mm) in `PHYSICAL`/`HYBRID` modes. Required alongside
    /// `stripHeightMm`.
    #[ serde( rename = "defaultFontSizeMm", default ) ]
    pub default_font_size_mm : Option< f32 >,
    /// Smallest font height (mm) `HYBRID` mode may shrink to before overflowing text is
    /// rejected. Optional; if unset, `HYBRID` shrinks text to fit the strip width with no floor.
    #[ serde( rename = "minFontSizeMm", default ) ]
    pub min_font_size_mm : Option< f32 >,
    /// Fixed font height as a fraction of canvas height, used by `RELATIVE` mode when set.
    /// Optional; if unset, `RELATIVE` auto-fits the largest size that fits the padded bounds.
    #[ serde( rename = "fontSizeRatio", default ) ]
    pub font_size_ratio : Option< f32 >,
    /// Horizontal text alignment within the padded strip.
    #[ serde( default = "default_align" ) ]
    pub align : TextAlign,
  }

  fn default_texture_height() -> u32 { 256 }
  fn default_padding() -> f32 { 0.1 }
  fn default_engraving_strength() -> f32 { 1.0 }
  fn default_engraving_roughness() -> f32 { 0.7 }
  fn default_engraving_darkening() -> f32 { 0.35 }
  fn default_align() -> TextAlign { TextAlign::Center }

  /// Text-sizing strategy for [`EngravingNodeConfig`] — see
  /// [`EngravingNodeConfig::resolved_sizing_mode`] for how it interacts with the physical /
  /// relative sizing fields.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq, Deserialize ) ]
  #[ serde( rename_all = "UPPERCASE" ) ]
  pub enum SizingMode
  {
    /// Font size is fixed by `defaultFontSizeMm / stripHeightMm`, constant regardless of text
    /// length. Overflowing the strip width is rejected (`EngravingCanvasError::TextOverflow`)
    /// rather than silently shrunk.
    Physical,
    /// Font size starts at `defaultFontSizeMm / stripHeightMm` and shrinks proportionally down
    /// to `minFontSizeMm` (if set) to fit the strip width before overflow is rejected.
    Hybrid,
    /// Font size is either `fontSizeRatio * canvasHeight` (if set) or auto-fit to the largest
    /// size that fits the padded canvas bounds — the original, unscaled behavior.
    Relative,
  }

  /// Horizontal text alignment within the padded engraving strip.
  #[ derive( Debug, Clone, Copy, PartialEq, Eq, Deserialize ) ]
  #[ serde( rename_all = "lowercase" ) ]
  pub enum TextAlign
  {
    /// Text starts at the left padding edge.
    Left,
    /// Text is centered between the padding edges (the original, unconfigurable behavior).
    Center,
    /// Text ends at the right padding edge.
    Right,
  }

  impl TextAlign
  {
    /// The `CanvasRenderingContext2d::set_text_align` value for this alignment.
    pub fn css( self ) -> &'static str
    {
      match self
      {
        Self::Left => "left",
        Self::Center => "center",
        Self::Right => "right",
      }
    }
  }

  /// The subset of [`EngravingNodeConfig`] that's safe to change after
  /// [`crate::webgl::engraving::EngravingSession::build`] has already allocated a node's
  /// offscreen canvas and GPU mask texture, without desyncing either of them.
  ///
  /// `node_name`, `uv_channel`, `aspect_ratio` and `texture_height` are deliberately *not*
  /// present here: they determine the canvas/texture dimensions and UV binding baked in at
  /// `build` time, and this type has no field to hold them through, so mutating them via
  /// [`crate::webgl::engraving::EngravingSession::text_layout_mut`] is a compile error rather
  /// than a runtime footgun. Everything that *is* here only affects how
  /// [`crate::webgl::engraving::EngravingCanvasManager::render_text_mip_chain`] lays text out
  /// inside that already-fixed canvas, which is always safe to redo.
  #[ derive( Debug, Clone, Copy ) ]
  pub struct EngravingTextLayout
  {
    /// Upper bound on text length, enforced before rasterizing.
    pub max_characters : usize,
    /// Fractional padding (of the shorter canvas dimension) kept empty around the text on every side.
    pub padding : f32,
    /// Horizontal text alignment within the padded strip.
    pub align : TextAlign,
    /// Explicit text-sizing strategy — see [`Self::resolved_sizing_mode`].
    pub sizing_mode : Option< SizingMode >,
    /// Physical height (mm) of the `TEXCOORD_1` engraving strip on the real object.
    pub strip_height_mm : Option< f32 >,
    /// Target physical font height (mm) in `PHYSICAL`/`HYBRID` modes.
    pub default_font_size_mm : Option< f32 >,
    /// Smallest font height (mm) `HYBRID` mode may shrink to before overflowing text is rejected.
    pub min_font_size_mm : Option< f32 >,
    /// Fixed font height as a fraction of canvas height, used by `RELATIVE` mode when set.
    pub font_size_ratio : Option< f32 >,
  }

  impl EngravingTextLayout
  {
    /// Resolves the effective [`SizingMode`] — see [`EngravingNodeConfig::resolved_sizing_mode`],
    /// which this mirrors.
    pub fn resolved_sizing_mode( &self ) -> SizingMode
    {
      if let Some( mode ) = self.sizing_mode
      {
        return mode;
      }
      if self.strip_height_mm.is_some() && self.default_font_size_mm.is_some()
      {
        SizingMode::Hybrid
      }
      else
      {
        SizingMode::Relative
      }
    }
  }

  impl EngravingNodeConfig
  {
    /// Returns the offscreen canvas / texture size `( width, height )` implied by `texture_height` and `aspect_ratio`.
    pub fn texture_size( &self ) -> ( u32, u32 )
    {
      let width = ( self.texture_height as f32 * self.aspect_ratio ).round().max( 1.0 ) as u32;
      ( width, self.texture_height )
    }

    /// Resolves the effective [`SizingMode`]: `sizing_mode` if set explicitly, otherwise
    /// `HYBRID` when both `strip_height_mm` and `default_font_size_mm` are present, otherwise
    /// `RELATIVE` — so a config with no physical fields at all falls back to the original
    /// auto-fit behavior without any explicit opt-in.
    pub fn resolved_sizing_mode( &self ) -> SizingMode
    {
      self.text_layout().resolved_sizing_mode()
    }

    /// Extracts the subset of this config that's safe to change after
    /// [`crate::webgl::engraving::EngravingSession::build`] — see [`EngravingTextLayout`].
    pub fn text_layout( &self ) -> EngravingTextLayout
    {
      EngravingTextLayout
      {
        max_characters : self.max_characters,
        padding : self.padding,
        align : self.align,
        sizing_mode : self.sizing_mode,
        strip_height_mm : self.strip_height_mm,
        default_font_size_mm : self.default_font_size_mm,
        min_font_size_mm : self.min_font_size_mm,
        font_size_ratio : self.font_size_ratio,
      }
    }

    fn validate( &self ) -> Result< (), EngravingConfigError >
    {
      if self.node_name.is_empty()
      {
        return Err( EngravingConfigError( "nodeName must not be empty".into() ) );
      }
      if self.uv_channel > 4
      {
        return Err( EngravingConfigError( format!( "uvChannel must be in 0..=4, got {}", self.uv_channel ) ) );
      }
      if self.aspect_ratio <= 0.0
      {
        return Err( EngravingConfigError( format!( "aspectRatio must be positive, got {}", self.aspect_ratio ) ) );
      }
      if self.max_characters == 0
      {
        return Err( EngravingConfigError( "maxCharacters must be at least 1".into() ) );
      }
      if self.allowed_fonts.is_empty()
      {
        return Err( EngravingConfigError( format!( "node '{}': allowedFonts must not be empty", self.node_name ) ) );
      }
      if !self.allowed_fonts.iter().any( | f | &**f == &*self.default_font )
      {
        return Err( EngravingConfigError( format!( "node '{}': defaultFont '{}' is not listed in allowedFonts", self.node_name, self.default_font ) ) );
      }
      if !( 0.0..=0.49 ).contains( &self.padding )
      {
        return Err( EngravingConfigError( format!( "padding must be in 0.0..=0.49, got {}", self.padding ) ) );
      }
      if let Some( mode ) = self.sizing_mode
      {
        if matches!( mode, SizingMode::Physical | SizingMode::Hybrid )
        && ( self.strip_height_mm.is_none() || self.default_font_size_mm.is_none() )
        {
          return Err( EngravingConfigError( format!(
            "node '{}': sizingMode {mode:?} requires both stripHeightMm and defaultFontSizeMm", self.node_name
          ) ) );
        }
      }
      if let Some( h ) = self.strip_height_mm
      {
        if h <= 0.0
        {
          return Err( EngravingConfigError( format!( "node '{}': stripHeightMm must be positive, got {}", self.node_name, h ) ) );
        }
      }
      if let Some( f ) = self.default_font_size_mm
      {
        if f <= 0.0
        {
          return Err( EngravingConfigError( format!( "node '{}': defaultFontSizeMm must be positive, got {}", self.node_name, f ) ) );
        }
      }
      if let Some( min_mm ) = self.min_font_size_mm
      {
        if min_mm <= 0.0
        {
          return Err( EngravingConfigError( format!( "node '{}': minFontSizeMm must be positive, got {}", self.node_name, min_mm ) ) );
        }
        if let Some( default_mm ) = self.default_font_size_mm
        {
          if min_mm > default_mm
          {
            return Err( EngravingConfigError( format!(
              "node '{}': minFontSizeMm ({min_mm}) must not exceed defaultFontSizeMm ({default_mm})", self.node_name
            ) ) );
          }
        }
      }
      if let Some( ratio ) = self.font_size_ratio
      {
        if !( 0.0..=1.0 ).contains( &ratio ) || ratio <= 0.0
        {
          return Err( EngravingConfigError( format!( "node '{}': fontSizeRatio must be in (0.0, 1.0], got {}", self.node_name, ratio ) ) );
        }
      }
      Ok( () )
    }
  }
}

crate::mod_interface!
{
  orphan use
  {
    EngravingConfig,
    EngravingNodeConfig,
    EngravingConfigError,
    EngravingTextLayout,
    SizingMode,
    TextAlign,
  };
}
