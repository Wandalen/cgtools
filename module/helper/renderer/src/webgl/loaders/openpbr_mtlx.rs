mod private
{
  use crate::webgl::material::{ OpenPbrSurface, openpbr_input_apply };
  use quick_xml::Reader;
  use quick_xml::events::{ BytesStart, Event };

  /// Errors produced while reading an OpenPBR surface from a MaterialX (`.mtlx`)
  /// document.
  #[ derive( Debug ) ]
  pub enum MtlxError
  {
    /// XML could not be parsed ( message from the underlying reader ).
    Xml( String ),
    /// An `<input>` targeted a scalar OpenPBR parameter with a `float` /
    /// `color3` / `boolean` type token but carried an unparsable value — a real
    /// content defect, surfaced instead of silently mis-applied.
    Input
    {
      /// The `name` attribute of the offending input.
      name : String,
      /// The `type` attribute of the offending input.
      type_name : String,
      /// The `value` attribute of the offending input.
      value : String,
    },
  }

  impl std::fmt::Display for MtlxError
  {
    fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
    {
      match self
      {
        MtlxError::Xml( message ) => write!( f, "malformed .mtlx XML: {message}" ),
        MtlxError::Input { name, type_name, value } =>
        {
          write!( f, "unparsable OpenPBR input '{name}' ( type '{type_name}', value '{value}' )" )
        }
      }
    }
  }

  impl std::error::Error for MtlxError {}

  /// True for the type tokens this reader treats as scalar OpenPBR parameters.
  /// Anything else ( `filename`, `vector2`/`vector3`, `integer`, … ) is a
  /// texture or geometry input that the texture plumbing step owns.
  fn is_scalar_type( type_name : &str ) -> bool
  {
    matches!( type_name, "float" | "color3" | "boolean" )
  }

  /// Applies one `<input>` element's `name`/`type`/`value` attributes to an
  /// in-progress surface. Only literal scalar values are applied: an input
  /// wired to an upstream node ( `nodename` attribute, no `value` ) or left
  /// unset is skipped here — its value is driven by the texture/graph step
  /// ( §3.1 ) or by the nodedef default, which the surface already starts
  /// from. A *present* scalar value that cannot be parsed is a hard error.
  fn apply_input( surface : &mut OpenPbrSurface, start : &BytesStart< '_ > ) -> Result< (), MtlxError >
  {
    let mut name = String::new();
    let mut type_name = String::new();
    let mut value = String::new();
    for attr in start.attributes()
    {
      let attr = attr.map_err( | err | MtlxError::Xml( err.to_string() ) )?;
      match attr.key.local_name().as_ref()
      {
        b"name" => name = String::from_utf8_lossy( attr.value.as_ref() ).into_owned(),
        b"type" => type_name = String::from_utf8_lossy( attr.value.as_ref() ).into_owned(),
        b"value" => value = String::from_utf8_lossy( attr.value.as_ref() ).into_owned(),
        _ => {}
      }
    }
    if is_scalar_type( &type_name )
    {
      let value = value.trim();
      if value.is_empty()
      {
        // Connected ( nodename ) or unset input — resolve via defaults / the
        // texture step, never mis-apply an empty string.
        return Ok( () );
      }
      let applied = openpbr_input_apply( surface, &name, &type_name, value );
      if !applied
      {
        return Err( MtlxError::Input { name, type_name, value : value.to_string() } );
      }
    }
    Ok( () )
  }

  /// Extracts every `open_pbr_surface` node from a MaterialX document.
  ///
  /// A surface is an element whose tag name equals `open_pbr_surface`
  /// ( MaterialX node-instance naming ) with `<input name type value/>`
  /// children; parameters absent from the document keep their spec defaults,
  /// so parsing starts from [`OpenPbrSurface::spec_default`] and applies only
  /// the present inputs. Multiple surfaces ( several materials in one file )
  /// are returned in document order. Pure XML/string work — no GPU, no texture
  /// resolution — so it is natively unit-testable.
  ///
  /// # Errors
  ///
  /// Returns [`MtlxError::Xml`] for malformed XML and [`MtlxError::Input`]
  /// when a scalar OpenPBR input cannot be parsed ( wrong value syntax or
  /// wrong type token for a known scalar parameter ).
  pub fn openpbr_surfaces_from_mtlx( xml : &str ) -> Result< Vec< OpenPbrSurface >, MtlxError >
  {
    let mut reader = Reader::from_str( xml );
    reader.config_mut().trim_text( true );

    let mut surfaces : Vec< OpenPbrSurface > = Vec::new();
    let mut surface : Option< OpenPbrSurface > = None;

    let mut buf = Vec::new();
    loop
    {
      match reader.read_event_into( &mut buf ).map_err( | e | MtlxError::Xml( e.to_string() ) )?
      {
        Event::Start( e ) =>
        {
          match e.local_name().as_ref()
          {
            b"open_pbr_surface" => surface = Some( OpenPbrSurface::spec_default() ),
            b"input" =>
            {
              if let Some( current ) = surface.as_mut()
              {
                apply_input( current, &e )?;
              }
            }
            _ => {}
          }
        }
        // `<input … />` is self-closing in MaterialX, so it arrives as Empty,
        // not Start followed by End.
        Event::Empty( e ) =>
        {
          match e.local_name().as_ref()
          {
            b"open_pbr_surface" => surfaces.push( OpenPbrSurface::spec_default() ),
            b"input" =>
            {
              if let Some( current ) = surface.as_mut()
              {
                apply_input( current, &e )?;
              }
            }
            _ => {}
          }
        }
        Event::End( e ) =>
        {
          if e.local_name().as_ref() == b"open_pbr_surface"
          {
            if let Some( s ) = surface.take()
            {
              surfaces.push( s );
            }
          }
        }
        Event::Eof => break,
        _ => {}
      }
      buf.clear();
    }

    Ok( surfaces )
  }
}

crate::mod_interface!
{
  own use
  {
    openpbr_surfaces_from_mtlx,
    MtlxError
  };
}
