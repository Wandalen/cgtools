//! This module provides functionality for loading UFO
//! fonts and converting text into a 3D mesh representation.


// Fix(TASK-021)
// Root cause : this module was gated on `text` while its glyph pipeline calls
// `contours_to_fill_geometry`, which only exists under `font-processing` — so
// `--features text` alone could never compile (surfaced by TASK-055).
// Pitfall : gate a module on the feature that provides everything it calls,
// not on the broadest feature it is thematically related to.
#[ cfg( feature = "font-processing" ) ]
mod private
{
  use rustc_hash::FxHashMap;
  use std::str::FromStr;
  use kurbo::flatten;
  use mingl::geometry::BoundingBox;
  use norad::{ PointType, ContourPoint, Contour };
  use minwebgl as gl;
  use gl::F32x3;
  use quick_xml::{ Reader, events::Event };
  use crate::
  {
    PrimitiveData,
    Transform,
    contours_to_fill_geometry
  };

  /// Represents a single character glyph, including its contours and a generated 3D body.
  #[ derive( Clone ) ]
  pub struct Glyph
  {
    /// The character associated with the glyph.
    _character : char,
    /// A vector of contours, where each contour is a vector of 2D points.
    contours : Vec< Vec< [ f32; 2 ] > >,
    /// The generated 3D primitive data for the glyph's body.
    body : Option< PrimitiveData >,
    /// The bounding box of the glyph.
    bounding_box : BoundingBox
  }

  impl Glyph
  {
    /// Creates a new `Glyph` from a vector of 2D contours and a character.
    fn new( contours : Vec< Vec< [ f64; 2 ] > >, character : char ) -> Self
    {
      let mut contours = contours.into_iter()
      .map
      (
        | v |
        v.into_iter()
        .map
        (
          | [ a, b ] | [ a as f32, b as f32 ]
        )
        .collect::< Vec< _ > >()
      )
      .collect::< Vec< _ > >();

      let flat_contours = contours.iter().flatten().flatten().copied().collect::< Vec< _ > >();
      let bounding_box = BoundingBox::compute2d( &flat_contours );

      let [ x1, y1 ] = [ bounding_box.left(), bounding_box.down() ];
      let [ x2, y2 ] = [ bounding_box.right(), bounding_box.up() ];

      let half_x = ( x2 - x1 ) / 2.0;
      let half_y = ( y2 - y1 ) / 2.0;
      let offset_x = x1;
      let offset_y = y1;
      let offset_x = - half_x - offset_x;
      let offset_y = - half_y - offset_y;

      for contour in &mut contours
      {
        for point in contour.iter_mut()
        {
          point[ 0 ] += offset_x;
          point[ 1 ] += offset_y;
        }
      }

      let bounding_box = BoundingBox::new
      (
        [ ( x1 + offset_x ), ( y1 + offset_y ), 0.0 ],
        [ ( x2 + offset_x ), ( y2 + offset_y ), 0.0 ]
      );

      Self
      {
        _character : character,
        contours,
        body : None,
        bounding_box
      }
    }

    /// Scales the glyph's contours and bounding box by a given factor.
    fn scale( &mut self, scale : f32)
    {
      let [ x1, y1 ] = [ self.bounding_box.left(), self.bounding_box.down() ];
      let [ x2, y2 ] = [ self.bounding_box.right(), self.bounding_box.up() ];

      for contour in &mut self.contours
      {
        for point in contour.iter_mut()
        {
          point[ 0 ] *= scale;
          point[ 1 ] *= scale;
        }
      }

      self.bounding_box.min = [ x1 * scale, y1 * scale, 0.0 ].into();
      self.bounding_box.max = [ x2 * scale, y2 * scale, 0.0 ].into();
    }

    /// Returns the glyph's flattened contours, each a sequence of 2D points.
    #[ must_use ]
    pub fn contours( &self ) -> &[ Vec< [ f32; 2 ] > ]
    {
      &self.contours
    }

    /// Creates a `Glyph` from a `.glif` file's byte data.
    ///
    /// # Panics
    ///
    /// Panics if `glif_bytes` is not valid UTF-8.
    #[ expect( clippy::too_many_lines, reason = "the glif XML event loop is one linear state machine; splitting it into helpers would scatter the per-event state transitions without shrinking the logic" ) ]
    #[ must_use ]
    pub fn from_glif( glif_bytes : &[ u8 ], character : char ) -> Option< Self >
    {
      let glif_str = std::str::from_utf8( glif_bytes ).unwrap();
      let mut reader = Reader::from_str( glif_str );
      reader.config_mut().trim_text( true );

      let mut raw_contours = vec![];
      let mut contour_points = vec![];

      loop
      {
        let event = reader.read_event();
        match event
        {
          Ok( Event::Empty( e ) ) if e.starts_with( b"point" ) =>
          {
            let element = e.clone();

            let mut x = None;
            let mut y = None;
            let smooth = true;
            // Fix(BUG-215)
            // Root cause: `typ` was declared once per *contour* (outside this loop),
            // so a point with no explicit `type` attribute -- the normal, spec-correct
            // way to encode an off-curve bezier control point in UFO/glif -- silently
            // inherited whatever type the *previous* point in the same contour had,
            // instead of defaulting to `OffCurve`. Confirmed against `norad` 0.18.4's
            // own reference parser (`glyph/parse.rs::parse_point`), which declares
            // `let mut typ = PointType::OffCurve;` fresh inside its own per-point
            // function, never carried over between points.
            // Pitfall: a state-machine accumulator that must reset per-iteration needs
            // its `let mut` *inside* the loop body at the right granularity -- placing
            // it outside silently widens its lifetime to the next coarser loop level.
            let mut typ = PointType::OffCurve;

            for attr in element.attributes()
            {
              let Ok( attr ) = attr
              else
              {
                continue;
              };

              let Ok( value ) = String::from_utf8( attr.value.to_vec() )
              else
              {
                continue;
              };

              match attr.key.0
              {
                b"x" => x = value.parse::< f64 >().ok(),
                b"y" => y = value.parse::< f64 >().ok(),
                // Fix(BUG-128)
                // Root cause: the UFO/glif spec's point element attribute is named
                // `type` (confirmed against `norad` 0.18.4's own glif parser, which
                // reads exactly `b"type"`), but this match arm looked for `b"typ"` --
                // a one-letter typo that can never match a real `.glif` file, so
                // every point silently kept the loop's `PointType::Move` default.
                // Pitfall: an unmatched byte-string arm in a `match` with a `_ => {}`
                // catch-all fails silently -- it never panics or errors, it just never
                // fires. Cross-check attribute names against the format spec or a
                // reference parser, not just internal self-consistency.
                b"type" =>
                {
                  let Ok( t ) = PointType::from_str( &value )
                  else
                  {
                    continue;
                  };
                  typ = t;
                }
                _ => {}
              }
            }

            if x.is_none() || y.is_none()
            {
              continue;
            }

            contour_points.push
            (
              ContourPoint::new
              (
                x.unwrap(),
                y.unwrap(),
                typ,
                smooth,
                None,
                None
              )
            );
          },
          Ok( Event::End( e ) ) if e.starts_with( b"contour" ) =>
          {
            let mut contour = Contour::default();
            contour.points = std::mem::take(&mut contour_points);
            raw_contours.push( contour );
          },
          Ok( Event::Eof ) => break,
          _ => ()
        }
      }

      let mut contours = vec![];
      let mut curves = vec![];

      for contour in raw_contours
      {
        let mut path = vec![];
        let Ok( bez_path ) = contour.to_kurbo()
        else
        {
          return None;
        };

        flatten
        (
          bez_path.elements().iter().copied(),
          0.25,
          | p | path.push( p )
        );

        let mut contour = vec![];

        for p in &path
        {
          match p
          {
            kurbo::PathEl::MoveTo( point ) |
            kurbo::PathEl::LineTo( point ) => contour.push( [ point.x, point.y ] ),
            kurbo::PathEl::ClosePath =>
            {
              contours.push( contour.clone() );
              contour.clear();
            },
            _ => ()
          }
        }

        curves.push( bez_path );
        contours.push( contour );
      }

      contours.retain( | c | !c.is_empty() );

      if contours.is_empty()
      {
        return None;
      }

      Some( Glyph::new( contours, character ) )
    }
  }

  /// Computes the per-glyph scale factor that rescales a font's tallest glyph to
  /// `target_scale` units.
  ///
  /// `max_y` is guarded away from `0.0` so a font whose glyphs are all
  /// zero-height (or a font with no glyphs at all, where `max_y` never leaves
  /// its `0.0` seed) yields a finite scale factor instead of `Infinity`.
  // Fix(BUG-500)
  // Root cause: `Font::new` divided `scale / max_y` with no guard against
  // `max_y == 0.0` -- reachable whenever every loaded glyph is zero-height, or
  // there are zero glyphs (the pre-loop `max_y` seed of `0.0` then never gets
  // raised). Rust float division by zero doesn't panic; it silently produces
  // `Infinity`, which every subsequent `glyph.scale( ... )` call then
  // multiplies every glyph coordinate by, poisoning them to `Infinity`/`NaN`.
  // Pitfall: a "max of measured values" seeded at `0.0` looks like a safe
  // default, but it is only safe for the max-tracking loop itself -- the
  // *result* of that loop being used as a divisor afterward re-introduces
  // exactly the zero/near-zero case the seed was chosen to tolerate.
  #[ must_use ]
  pub fn glyph_rescale_factor( target_scale : f32, max_y : f32 ) -> f32
  {
    target_scale / max_y.max( f32::EPSILON )
  }

  /// Represents a font loaded from UFO files, containing a collection of glyphs.
  #[ derive( Clone ) ]
  pub struct Font
  {
    /// A map of characters to their corresponding glyphs.
    glyphs : FxHashMap< char, Glyph >,
    /// The maximum bounding box of glyph in the font.
    max_size : BoundingBox
  }

  impl Font
  {
    /// Returns the union bounding box of every glyph in the font.
    #[ must_use ]
    pub fn max_size( &self ) -> BoundingBox
    {
      self.max_size
    }

    /// Builds a `Font` directly from pre-built glyphs, computing `max_size` as the
    /// union of each glyph's own bounding box (mirroring `Font::new`'s union-box
    /// step). Unlike `Font::new`, this skips the UFO-loading pipeline's automatic
    /// rescale-to-a-common-em-size step -- primarily useful for constructing
    /// synthetic fonts (e.g. in tests) from glyphs built via `Glyph::from_glif`.
    #[ must_use ]
    pub fn from_glyphs( glyphs : impl IntoIterator< Item = ( char, Glyph ) > ) -> Self
    {
      let glyphs : FxHashMap< char, Glyph > = glyphs.into_iter().collect();

      let mut min = F32x3::MAX;
      let mut max = F32x3::MIN;
      for glyph in glyphs.values()
      {
        // Fix(BUG-216)
        // Root cause: `Vector`'s `<`/`>` operators route through its `PartialOrd`/`Ord`
        // impls, which delegate to `[E; N]`'s lexicographic array comparison (compares
        // the x component first, only inspecting y/z to break an x-tie) -- not the
        // component-wise per-axis min/max an AABB union needs. Confirmed against
        // `Vector::min`/`Vector::max` (`ndarray_cg::vector::arithmetics`), the correct
        // component-wise methods already used by this exact dependency's own
        // `BoundingBox::compute`/`compute2d`.
        // Pitfall: a `Vector` supports two unrelated orderings -- a total, lexicographic
        // one (via `<`/`>`/`Ord`, useful for e.g. canonical sort keys) and a
        // component-wise one (via `.min()`/`.max()`, useful for geometry) -- picking the
        // operator instead of the method silently selects the wrong one for AABB math.
        min = min.min( glyph.bounding_box.min );
        max = max.max( glyph.bounding_box.max );
      }

      Self
      {
        glyphs,
        max_size : BoundingBox::new( min, max )
      }
    }

    /// Asynchronously loads a new `Font` from a UFO directory path.
    //
    // Fix: the 3 glyph-fetch loops below used `.expect(...)`, panicking the
    // entire app if ANY of the full a-z/A-Z/0-9 set (62 files) was absent from
    // a UFO directory, even though callers only ever render a handful of
    // specific letters.
    // Root cause: `.expect` treated a missing-but-optional glyph the same as a
    // fatal load error -- a font legitimately may not define every glyph.
    // Pitfall: a loop over an exhaustive enumeration (full alphabet/digit set)
    // must tolerate individual misses via `continue`; only the per-glyph fetch
    // is optional, not the overall load.
    async fn new( path : &str ) -> Self
    {
      let mut glyphs = FxHashMap::< char, Glyph >::default();
      let glyphs_path = path.to_string() + "/glyphs";

      for c in b'a'..=b'z'
      {
        let glyph_path = format!( "{}/{}.glif", glyphs_path, c as char );
        let Ok( glif_bytes ) = gl::file::load( &glyph_path ).await
        else
        {
          continue;
        };
        if let Some( glyph ) = Glyph::from_glif( &glif_bytes, c as char )
        {
          glyphs.insert( c as char, glyph );
        }
      }

      for c in b'A'..=b'Z'
      {
        let glyph_path = format!( "{}/{}_.glif", glyphs_path, c as char );
        let Ok( glif_bytes ) = gl::file::load( &glyph_path ).await
        else
        {
          continue;
        };
        if let Some( glyph ) = Glyph::from_glif( &glif_bytes, c as char )
        {
          glyphs.insert( c as char, glyph );
        }
      }

      for ( c, name ) in
      [
        ( '0', "zero" ),
        ( '1', "one" ),
        ( '2', "two" ),
        ( '3', "three" ),
        ( '4', "four" ),
        ( '5', "five" ),
        ( '6', "six" ),
        ( '7', "seven" ),
        ( '8', "eight" ),
        ( '9', "nine" )
      ]
      {
        let glyph_path = format!( "{glyphs_path}/{name}.glif" );
        let Ok( glif_bytes ) = gl::file::load( &glyph_path ).await
        else
        {
          continue;
        };
        if let Some( glyph ) = Glyph::from_glif( &glif_bytes, c )
        {
          glyphs.insert( c, glyph );
        }
      }

      // Fix(UX-DX-7)
      // Root cause: if every per-glyph load above missed (bad `path`, entirely
      // empty font directory, etc.), `glyphs` ends up empty with zero
      // diagnostic signal -- the function still returns a `Self` that looks
      // like a legitimate (if sparse) font, indistinguishable from a
      // legitimately-partial one.
      // Pitfall: a loop that tolerates individual misses via `continue` (see
      // the Fix(TASK-0xx) note above each loading loop) must still surface
      // the all-missed case -- tolerating every individual failure silently
      // is not the same as tolerating total failure silently.
      if glyphs.is_empty()
      {
        web_sys::console::warn_1( &format!( "UFO font at \"{path}\" loaded zero glyphs -- check the path and glyph file names" ).into() );
      }

      let [ mut max_x, mut max_y ] = [ 0.0, 0.0 ];
      for glyph in glyphs.values()
      {
        let [ x1, y1 ] = [ glyph.bounding_box.left(), glyph.bounding_box.down() ];
        let [ x2, y2 ] = [ glyph.bounding_box.right(), glyph.bounding_box.up() ];
        let x = x2 - x1;
        let y = y2 - y1;
        if max_x < x
        {
          max_x = x;
        }
        if max_y < y
        {
          max_y = y;
        }
      }

      let scale = 250.0;
      for glyph in glyphs.values_mut()
      {
        glyph.scale( glyph_rescale_factor( scale, max_y ) );
      }

      let mut min = F32x3::MAX;
      let mut max = F32x3::MIN;
      for glyph in glyphs.values()
      {
        // Fix(BUG-216)
        // Root cause: `Vector`'s `<`/`>` operators route through its `PartialOrd`/`Ord`
        // impls, which delegate to `[E; N]`'s lexicographic array comparison (compares
        // the x component first, only inspecting y/z to break an x-tie) -- not the
        // component-wise per-axis min/max an AABB union needs. Confirmed against
        // `Vector::min`/`Vector::max` (`ndarray_cg::vector::arithmetics`), the correct
        // component-wise methods already used by this exact dependency's own
        // `BoundingBox::compute`/`compute2d`.
        // Pitfall: a `Vector` supports two unrelated orderings -- a total, lexicographic
        // one (via `<`/`>`/`Ord`, useful for e.g. canonical sort keys) and a
        // component-wise one (via `.min()`/`.max()`, useful for geometry) -- picking the
        // operator instead of the method silently selects the wrong one for AABB math.
        min = min.min( glyph.bounding_box.min );
        max = max.max( glyph.bounding_box.max );
      }

      for glyph in glyphs.values_mut()
      {
        glyph.body = contours_to_fill_geometry( &glyph.contours );
      }

      Self
      {
        glyphs,
        max_size : BoundingBox::new( min, max )
      }
    }
  }

  /// Asynchronously loads multiple fonts from a list of font names.
  pub async fn fonts_load( font_names : &[ &str ] ) -> FxHashMap< String, Font >
  {
    let mut fonts = FxHashMap::< String, Font >::default();

    for font_name in font_names
    {
      let font_path = format!( "static/fonts/ufo/{font_name}.ufo" );
      fonts.insert( (*font_name).to_string(), Font::new( &font_path ).await );
    }

    fonts
  }

  /// Computes the `( glyph, placement )` pairs for every character in `text` that has a
  /// loaded glyph in `font`, using the two-pass centered-advance layout shared by
  /// `text_to_mesh` and `text_to_countour_mesh`. Those two callers differ only in how
  /// they turn each placed glyph into geometry (filled body vs. outlined contours) --
  /// this function owns the layout math both need identically.
  // Fix(UX-DX-8)
  // Root cause: `text_to_mesh` and `text_to_countour_mesh` each carried their own
  // copy of the identical two-pass advance/centering logic (pass 1: pre-compute the
  // starting offset; pass 2: advance-place-advance per glyph), diverging only in the
  // final geometry-generation step. Any future fix to the shared layout math (as
  // BUG-129 already had to be, twice, in both copies) risked being applied to only
  // one copy.
  // Pitfall: two functions that read as "near-identical" during a bug fix are a
  // signal to consolidate, not a coincidence to fix twice -- duplicated logic drifts
  // the moment only one copy gets the next fix.
  #[ must_use ]
  fn glyph_placements( text : &str, font : &Font, transform : &Transform ) -> Vec< ( Glyph, Transform ) >
  {
    let start_transform = transform.clone();
    let mut transform = start_transform.clone();
    transform.scale = [ 0.003, 0.003, 1.0 ].into();
    let max_x = font.max_size.max[ 0 ] - font.max_size.min[ 0 ];
    let max_y = font.max_size.max[ 1 ] - font.max_size.min[ 1 ];
    let half_x = max_x * transform.scale[ 0 ];

    for char in text.chars()
    {
      let Some( glyph ) = font.glyphs.get( &char )
      else
      {
        transform.translation[ 0 ] -= half_x / 2.0;
        continue;
      };

      let glyph_x = glyph.bounding_box.width() * transform.scale[ 0 ];
      transform.translation[ 0 ] -= if glyph_x < half_x / 4.0
      {
        half_x / 2.0
      }
      else
      {
        glyph_x / 2.0
      }
    }

    let mut placements = vec![];

    for char in text.chars()
    {
      let Some( glyph ) = font.glyphs.get( &char ).cloned()
      else
      {
        transform.translation[ 0 ] += half_x;
        continue;
      };

      let glyph_y = glyph.bounding_box.height();
      let diff = ( max_y - ( glyph_y * 0.5 ) ) * transform.scale[ 1 ];
      transform.translation[ 1 ] = start_transform.translation[ 1 ];
      transform.translation[ 1 ] -= diff;
      let glyph_x = glyph.bounding_box.width() * transform.scale[ 0 ];
      // Fix(BUG-129)
      // Root cause: this advanced by the glyph's *full* slot width before placing
      // it, expecting the next glyph's leading step to land it correctly -- but
      // pass 1 above only ever subtracts a HALF slot-width per glyph, so this
      // pass's full-width single step over-advances by exactly one half
      // slot-width per glyph, compounding across the string.
      // Pitfall: pass 1 and pass 2 must advance by symmetric half-steps around
      // each glyph's placement (step, place, step) to keep glyphs centered in
      // contiguous slots -- splitting the advance asymmetrically (a whole step
      // here, an implicit half step there) silently drifts every glyph after the
      // first.
      let step = if glyph_x < half_x / 4.0 { half_x / 2.0 } else { glyph_x / 2.0 };
      transform.translation[ 0 ] += step;

      placements.push( ( glyph, transform.clone() ) );

      transform.translation[ 0 ] += step;
    }

    placements
  }

  /// Converts text string into a collection of filled mesh primitives using the specified font.
  #[ must_use ]
  pub fn text_to_mesh( text : &str, font : &Font, transform : &Transform ) -> Vec< PrimitiveData >
  {
    glyph_placements( text, font, transform )
    .into_iter()
    .filter_map( | ( glyph, placement ) |
    {
      let mut geometry = glyph.body.clone()?;
      geometry.transform = placement;
      Some( geometry )
    } )
    .collect()
  }

  /// Converts text string into outlined contour meshes with specified line width.
  #[ must_use ]
  pub fn text_to_countour_mesh(
    text : &str,
    font : &Font,
    transform : &Transform,
    width : f32
  ) -> Vec< PrimitiveData >
  {
    glyph_placements( text, font, transform )
    .into_iter()
    .flat_map( | ( glyph, placement ) |
    {
      glyph.contours.into_iter()
      .filter_map( move | curve |
      {
        let mut geometry = crate::primitive::curve_to_geometry( &curve, width )?;
        geometry.transform = placement.clone();
        Some( geometry )
      } )
      .collect::< Vec< _ > >()
    } )
    .collect()
  }
}

// Without `font-processing` the UFO pipeline simply does not exist — the
// symbols are configured out (loud absence at compile time), matching how
// `contours_to_fill_geometry` is gated in `primitive.rs`. The former
// always-return-None/empty stubs were silent-failure machinery with drifted
// signatures and are gone (TASK-021).
#[ cfg( not( feature = "font-processing" ) ) ]
mod private
{
}

crate::mod_interface!
{
  #[ cfg( feature = "font-processing" ) ]
  orphan use
  {
    fonts_load,
    Glyph,
    Font,
    glyph_rescale_factor,
    text_to_mesh,
    text_to_countour_mesh
  };
}
