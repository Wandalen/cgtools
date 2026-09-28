//! Tint resolution for emitted sprites: named tints, the scene's global
//! tint, a layer's `TintBehaviour` and the final per-sprite composition
//! (layer alpha, per-instance tint).

mod private
{
  use crate::compile::error::CompileError;
  use crate::layer::{ LayerBehaviour, TintBehaviour };
  use crate::object::Object;
  use crate::resource::TintRef;
  use crate::scene::Scene;
  use crate::spec::RenderSpec;

  /// Multiply the alpha channel of a tint by a per-layer alpha factor.
  #[ inline ]
  #[ must_use ]
  pub fn tinted( [ r, g, b, a ] : [ f32; 4 ], alpha : f32 ) -> [ f32; 4 ]
  {
    [ r, g, b, a * alpha ]
  }

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
  /// the result is ready to multiply straight into a `Sprite.tint`. The tint's
  /// `mode` is not read: validation admits only `Multiply`.
  ///
  /// # Errors
  ///
  /// [`CompileError::UnresolvedRef`] when the id names no declared tint or
  /// the tint's colour is not `"#rrggbb"` / `"#rrggbbaa"`.
  pub fn resolve_tint_ref( spec : &RenderSpec, tint_ref : &TintRef ) -> Result< [ f32; 4 ], CompileError >
  {
    let id = &tint_ref.0;
    let tint = spec.tints.iter().find( | t | &t.id == id )
      .ok_or_else( || CompileError::UnresolvedRef
      {
        kind : "tint",
        id : id.clone(),
        context : "tint reference".into(),
      })?;
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

  /// Resolve the effective global tint, honouring `Scene`'s runtime override.
  ///
  /// # Errors
  ///
  /// Same as [`resolve_tint_ref`] for the selected tint.
  pub fn scene_global_tint_resolve( spec : &RenderSpec, scene : &Scene ) -> Result< [ f32; 4 ], CompileError >
  {
    let tint_ref = scene.global_tint().cloned().or_else( || spec.pipeline.global_tint.clone() );
    let Some( tint_ref ) = tint_ref else { return Ok( [ 1.0, 1.0, 1.0, 1.0 ] ); };
    resolve_tint_ref( spec, &tint_ref )
  }

  /// Resolve a layer's [`TintBehaviour`] into the base RGBA multiplier fed to
  /// [`final_tint`].
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
  pub fn layer_base_tint
  (
    global_tint : [ f32; 4 ],
    spec : &RenderSpec,
    object : &Object,
    behaviour : &LayerBehaviour,
  ) -> Result< [ f32; 4 ], CompileError >
  {
    match &behaviour.tint
    {
      TintBehaviour::None => Ok( global_tint ),
      TintBehaviour::Flat( tref ) =>
      {
        let c = resolve_tint_ref( spec, tref )?;
        Ok(
        [
          global_tint[ 0 ] * c[ 0 ],
          global_tint[ 1 ] * c[ 1 ],
          global_tint[ 2 ] * c[ 2 ],
          global_tint[ 3 ] * c[ 3 ],
        ])
      }
      TintBehaviour::Masked { .. } => Err( CompileError::UnsupportedBehaviour
      {
        object : object.id.clone(),
        behaviour : "Masked tint (not implemented — use Flat or remove the tint behaviour)",
      }),
    }
  }

  /// Compose the per-sprite tint as
  /// `base * layer_alpha (alpha-channel only) * instance_tint`, where `base`
  /// is the layer's resolved tint from [`layer_base_tint`].
  #[ inline ]
  #[ must_use ]
  pub fn final_tint( base : [ f32; 4 ], layer_alpha : f32, inst : Option< [ f32; 4 ] > ) -> [ f32; 4 ]
  {
    let [ gr, gg, gb, ga ] = base;
    let composed = [ gr, gg, gb, ga * layer_alpha ];
    match inst
    {
      None => composed,
      Some( [ ir, ig, ib, ia ] ) =>
      [
        composed[ 0 ] * ir,
        composed[ 1 ] * ig,
        composed[ 2 ] * ib,
        composed[ 3 ] * ia,
      ],
    }
  }

}

mod_interface::mod_interface!
{
  own use tinted;
  own use hex_rgba_parse;
  own use resolve_tint_ref;
  own use scene_global_tint_resolve;
  own use layer_base_tint;
  own use final_tint;
}
