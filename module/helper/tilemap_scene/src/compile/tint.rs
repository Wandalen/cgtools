//! Tint resolution for emitted sprites: named tints, the scene's global
//! tint, a layer's `TintBehaviour` and the final per-sprite composition
//! (layer alpha, per-instance tint).

mod private
{
  use crate::compile::error::CompileError;
  use crate::layer::{ LayerBehaviour, TintBehaviour };
  use crate::object::Object;
  use crate::resource::{ Tint, TintRef };
  use crate::scene::Scene;
  use crate::spec::RenderSpec;
  use rustc_hash::FxHashMap as HashMap;
  use tilemap_renderer::types::BlendMode;

  /// Parse a `"#rrggbb"` or `"#rrggbbaa"` colour string into linear-ish
  /// `[f32; 4]`. Returns `None` on malformed input — caller decides whether
  /// to error or fall back.
  #[ must_use ]
  pub fn hex_rgba_parse( s : &str ) -> Option< [ f32; 4 ] >
  {
    let s = s.strip_prefix( '#' )?;
    let hex_byte = | i : usize | u8::from_str_radix( s.get( i..i + 2 )?, 16 ).ok();
    match s.len()
    {
      6 => Some(
      [
        f32::from( hex_byte( 0 )? ) / 255.0,
        f32::from( hex_byte( 2 )? ) / 255.0,
        f32::from( hex_byte( 4 )? ) / 255.0,
        1.0,
      ]),
      8 => Some(
      [
        f32::from( hex_byte( 0 )? ) / 255.0,
        f32::from( hex_byte( 2 )? ) / 255.0,
        f32::from( hex_byte( 4 )? ) / 255.0,
        f32::from( hex_byte( 6 )? ) / 255.0,
      ]),
      _ => None,
    }
  }

  /// Resolve a named [`TintRef`] to a strength-blended multiplier `[r,g,b,a]`.
  ///
  /// `strength` interpolates the parsed colour towards identity `[1,1,1,1]`, so
  /// the result is ready to multiply straight into a `Sprite.tint`. `context`
  /// names the referencing site in an unresolved-id error and is only built
  /// on failure.
  ///
  /// # Errors
  ///
  /// [`CompileError::UnresolvedRef`] when the id names no declared tint or
  /// the tint's colour is not `"#rrggbb"` / `"#rrggbbaa"`;
  /// [`CompileError::UnsupportedTintMode`] when its `mode` is not `Multiply`.
  pub fn resolve_tint_ref
  (
    spec : &RenderSpec,
    tint_ref : &TintRef,
    context : impl FnOnce() -> String,
  ) -> Result< [ f32; 4 ], CompileError >
  {
    let id = &tint_ref.0;
    let tint = spec.tints.iter().find( | t | &t.id == id )
      .ok_or_else( || CompileError::UnresolvedRef
      {
        kind : "tint",
        id : id.clone(),
        context : context(),
      })?;
    tint_multiplier( tint )
  }

  /// Parses `tint.color` and blends it towards identity by `tint.strength`.
  ///
  /// A multiplier can only express `Multiply`; any other `mode` is an error
  /// rather than a silent multiply.
  fn tint_multiplier( tint : &Tint ) -> Result< [ f32; 4 ], CompileError >
  {
    if tint.mode != BlendMode::Multiply
    {
      return Err( CompileError::UnsupportedTintMode { tint : tint.id.clone(), mode : tint.mode } );
    }
    let [ r, g, b, a ] = hex_rgba_parse( &tint.color ).ok_or_else( || CompileError::UnresolvedRef
    {
      kind : "tint color",
      id : tint.color.clone(),
      context : format!( "tint {:?}", tint.id ),
    })?;
    let s = tint.strength.clamp( 0.0, 1.0 );
    Ok(
    [
      1.0 + s * ( r - 1.0 ),
      1.0 + s * ( g - 1.0 ),
      1.0 + s * ( b - 1.0 ),
      1.0 + s * ( a - 1.0 ),
    ])
  }

  /// The frame's tint state: the resolved global tint plus every declared
  /// tint resolved once, keyed by id. Every sprite-emitting pass gets its
  /// final tint from [`FrameTints::sprite_tint`], so no emit site can pass
  /// the bare global tint and drop the layer's own tint.
  ///
  /// `TintBehaviour::Flat` is looked up here for every emitted sprite;
  /// calling [`resolve_tint_ref`] instead would repeat a linear search over
  /// `spec.tints` and a colour parse per sprite. A tint that does not resolve
  /// (unparsable colour, non-`Multiply` mode) is stored as `None`, so looking
  /// it up falls back to [`resolve_tint_ref`] and reports exactly its error.
  #[ derive( Debug ) ]
  pub struct FrameTints< 'a >
  {
    spec : &'a RenderSpec,
    global : [ f32; 4 ],
    resolved : HashMap< &'a str, Option< [ f32; 4 ] > >,
  }

  impl< 'a > FrameTints< 'a >
  {
    /// Resolves the effective global tint (`Scene`'s runtime override, else
    /// `pipeline.global_tint`, else identity) and every tint `spec` declares.
    ///
    /// # Errors
    ///
    /// Same as [`resolve_tint_ref`] for the selected global tint.
    pub fn new( spec : &'a RenderSpec, scene : &Scene ) -> Result< Self, CompileError >
    {
      let global = match scene.global_tint().or( spec.pipeline.global_tint.as_ref() )
      {
        Some( tint_ref ) => resolve_tint_ref( spec, tint_ref, || "scene.global_tint / pipeline.global_tint".into() )?,
        None => [ 1.0, 1.0, 1.0, 1.0 ],
      };
      let mut resolved = HashMap::default();
      for tint in &spec.tints
      {
        // First declaration wins, like `resolve_tint_ref`'s `find`.
        resolved.entry( tint.id.as_str() ).or_insert_with( || tint_multiplier( tint ).ok() );
      }
      Ok( Self { spec, global, resolved } )
    }

    /// The final tint of one sprite: the layer's base tint (the global tint,
    /// times the `Flat` tint if the layer declares one) with the layer alpha
    /// folded into the alpha channel, times the instance tint.
    ///
    /// # Errors
    ///
    /// [`CompileError::UnsupportedBehaviour`] for a `Masked` layer tint;
    /// otherwise the errors of [`resolve_tint_ref`] for a `Flat` one.
    pub fn sprite_tint
    (
      &self,
      object : &Object,
      behaviour : &LayerBehaviour,
      inst : Option< [ f32; 4 ] >,
    ) -> Result< [ f32; 4 ], CompileError >
    {
      let [ r, g, b, a ] = self.layer_base_tint( object, behaviour )?;
      let composed = [ r, g, b, a * behaviour.alpha ];
      Ok( match inst
      {
        None => composed,
        Some( [ ir, ig, ib, ia ] ) =>
        [
          composed[ 0 ] * ir,
          composed[ 1 ] * ig,
          composed[ 2 ] * ib,
          composed[ 3 ] * ia,
        ],
      })
    }

    /// Resolve a layer's [`TintBehaviour`] into its base RGBA multiplier.
    /// Private on purpose: an emit site that took it directly would skip the
    /// layer alpha and instance tint that [`Self::sprite_tint`] folds in.
    ///
    /// - `None` → the global tint unchanged.
    /// - `Flat(ref)` → global tint multiplied by the named tint, so each layer
    ///   (e.g. a per-player region overlay) can be coloured independently.
    /// - `Masked` → rejected with [`CompileError::UnsupportedBehaviour`]; it is
    ///   not yet implemented and must not silently degrade to the global tint.
    ///
    /// # Errors
    ///
    /// [`CompileError::UnsupportedBehaviour`] for `Masked`; otherwise the
    /// errors of [`resolve_tint_ref`].
    fn layer_base_tint( &self, object : &Object, behaviour : &LayerBehaviour ) -> Result< [ f32; 4 ], CompileError >
    {
      let global = self.global;
      match &behaviour.tint
      {
        TintBehaviour::None => Ok( global ),
        TintBehaviour::Flat( tref ) =>
        {
          let c = match self.resolved.get( tref.0.as_str() )
          {
            Some( Some( c ) ) => *c,
            _ => resolve_tint_ref( self.spec, tref, || format!( "object {:?} layer tint", object.id ) )?,
          };
          Ok(
          [
            global[ 0 ] * c[ 0 ],
            global[ 1 ] * c[ 1 ],
            global[ 2 ] * c[ 2 ],
            global[ 3 ] * c[ 3 ],
          ])
        }
        TintBehaviour::Masked { .. } => Err( CompileError::UnsupportedBehaviour
        {
          object : object.id.clone(),
          behaviour : "Masked tint",
        }),
      }
    }
  }

}

mod_interface::mod_interface!
{
  own use hex_rgba_parse;
  own use resolve_tint_ref;
  own use FrameTints;
}
