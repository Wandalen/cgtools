mod private
{
  use minwebgl as gl;
  use gl::JsCast;
  use gl::wasm_bindgen::JsValue;
  use wasm_bindgen_futures::JsFuture;
  use crate::webgl::engraving::{ EngravingTextLayout, SizingMode, TextAlign };

  /// Error produced while loading a font or rasterizing engraving text.
  #[ derive( Debug ) ]
  pub enum EngravingCanvasError
  {
    /// The runtime has no `window`/`document` (not running in a browser tab).
    NoDom,
    /// Failed to create or configure the offscreen `<canvas>`.
    CanvasSetup( String ),
    /// `document.fonts.load()` rejected or the requested font family never became `FontFace.status == "loaded"`.
    FontLoad( String ),
    /// `text.chars().count()` exceeded the node's configured `max_characters`.
    TooManyCharacters
    {
      /// Number of characters in the offending string.
      got : usize,
      /// The configured limit.
      max : usize
    },
    /// A `CanvasRenderingContext2d` call returned a JS exception.
    Draw( String ),
    /// `text` doesn't fit within the padded strip width at the resolved font size —
    /// `SizingMode::Physical` rejects any overflow outright, `SizingMode::Hybrid` rejects it
    /// only once shrinking has bottomed out at `minFontSizeMm` (or has nothing left to shrink
    /// with, if `minFontSizeMm` isn't set).
    TextOverflow
    {
      /// Rendered width of `text` at the font size that was attempted, in canvas px.
      text_width_px : f32,
      /// Available width inside the padded strip, in canvas px.
      max_width_px : f32,
    },
  }

  impl std::fmt::Display for EngravingCanvasError
  {
    fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
    {
      match self
      {
        Self::NoDom => write!( f, "no window/document available" ),
        Self::CanvasSetup( e ) => write!( f, "failed to set up offscreen canvas: {e}" ),
        Self::FontLoad( e ) => write!( f, "failed to load font: {e}" ),
        Self::TooManyCharacters { got, max } => write!( f, "text has {got} characters, exceeds max_characters ({max})" ),
        Self::Draw( e ) => write!( f, "canvas 2d draw call failed: {e}" ),
        Self::TextOverflow { text_width_px, max_width_px } => write!(
          f, "text overflows the engraving strip: rendered width {text_width_px}px exceeds the available {max_width_px}px \
          (shorten the text, lower defaultFontSizeMm, or switch to HYBRID/RELATIVE sizing)"
        ),
      }
    }
  }

  impl std::error::Error for EngravingCanvasError {}

  impl From< JsValue > for EngravingCanvasError
  {
    fn from( v : JsValue ) -> Self
    {
      Self::Draw( format!( "{v:?}" ) )
    }
  }

  /// A rasterized engraving mask, ready to be uploaded to a WebGL texture.
  ///
  /// Pixels are tightly packed RGBA8, row-major, top row first (the layout returned by
  /// `CanvasRenderingContext2d::get_image_data`). Per the engraving convention, the text is
  /// pure white (`#FFFFFF`) on a solid black (`#000000`) background, so any single channel
  /// (the shader samples `.r`) is usable directly as an 8-bit mask: `0` outside the glyphs,
  /// `255` inside them.
  #[ derive( Debug, Clone ) ]
  pub struct EngravingBitmap
  {
    /// Width in pixels.
    pub width : u32,
    /// Height in pixels.
    pub height : u32,
    /// Tightly packed RGBA8 pixel data, `width * height * 4` bytes.
    pub pixels : Vec< u8 >,
  }

  /// Loads fonts and rasterizes engraving text onto an offscreen 2D canvas.
  ///
  /// One instance owns one detached `<canvas>` element sized to a single node's engraving
  /// zone; it is reused across repeated `render_text_mip_chain` calls (e.g. as the user types),
  /// so no DOM nodes or event listeners are created per keystroke.
  pub struct EngravingCanvasManager
  {
    canvas : web_sys::HtmlCanvasElement,
    ctx : web_sys::CanvasRenderingContext2d,
    width : u32,
    height : u32,
  }

  impl EngravingCanvasManager
  {
    /// Creates a manager with an offscreen canvas of the given pixel size.
    ///
    /// `width`/`height` normally come from [`crate::webgl::engraving::EngravingNodeConfig::texture_size`],
    /// so the rasterized bitmap matches the GPU texture's dimensions exactly and no scaling is
    /// needed on upload.
    pub fn new( width : u32, height : u32 ) -> Result< Self, EngravingCanvasError >
    {
      let window = web_sys::window().ok_or( EngravingCanvasError::NoDom )?;
      let document = window.document().ok_or( EngravingCanvasError::NoDom )?;

      let canvas = document.create_element( "canvas" )
      .map_err( | e | EngravingCanvasError::CanvasSetup( format!( "{e:?}" ) ) )?
      .dyn_into::< web_sys::HtmlCanvasElement >()
      .map_err( | _ | EngravingCanvasError::CanvasSetup( "created element is not a canvas".into() ) )?;

      canvas.set_width( width );
      canvas.set_height( height );

      // `true`: this context is read back via `get_image_data` on every `render_text_mip_chain`
      // call (once per mip level), not just once — hint that up front so the browser doesn't
      // have to migrate the canvas off the GPU-backed fast path after the fact (and warn about
      // it in the console).
      let ctx = gl::context::from_canvas_2d_with_options( &canvas, true )
      .map_err( | e | EngravingCanvasError::CanvasSetup( format!( "{e:?}" ) ) )?;

      Ok( Self { canvas, ctx, width, height } )
    }

    /// The underlying (detached, never appended to the DOM) canvas element, exposed for
    /// debugging/preview purposes (e.g. temporarily appending it to inspect the rasterized text).
    pub fn canvas( &self ) -> &web_sys::HtmlCanvasElement { &self.canvas }

    /// Width in pixels of the offscreen canvas.
    pub fn width( &self ) -> u32 { self.width }

    /// Height in pixels of the offscreen canvas.
    pub fn height( &self ) -> u32 { self.height }

    /// Waits for `family` to be ready for use, downloading it first if necessary.
    ///
    /// Mirrors the spec's `document.fonts.load(font, text)`: passing `sample_text` restricts
    /// (browser-dependently) which glyphs are guaranteed loaded, so pass the actual string
    /// that is about to be rendered.
    async fn ensure_font_loaded( &self, family : &str, sample_text : &str ) -> Result< (), EngravingCanvasError >
    {
      let window = web_sys::window().ok_or( EngravingCanvasError::NoDom )?;
      let document = window.document().ok_or( EngravingCanvasError::NoDom )?;
      let fonts = document.fonts();

      // A probe size doesn't matter for *which* family gets loaded, only the family name does;
      // the shorthand still must be syntactically valid CSS font shorthand.
      let font_shorthand = format!( "16px \"{family}\"" );

      let promise = fonts.load_with_text( &font_shorthand, sample_text );

      JsFuture::from( promise ).await
      .map_err( | e | EngravingCanvasError::FontLoad( format!( "{e:?}" ) ) )?;

      Ok( () )
    }

    /// Loads `font_family`, then rasterizes `text` once per mip level of the base `width` x
    /// `height` size (standard GL halving-with-floor down to `1` x `1`), returning one
    /// [`EngravingBitmap`] per level in order (index `0` is the full-size level).
    ///
    /// Each level is drawn from scratch at its own resolution rather than downsampled from
    /// level 0: for a binary black/white text mask, box-filtering a crisp high-res rasterization
    /// aliases badly at the glyph edges (and that aliasing feeds directly into the `dFdx`/`dFdy`
    /// relief-normal perturbation in `main.frag`, which is what actually sparkles under
    /// minification). Re-rasterizing at each level's true resolution lets the canvas 2D
    /// rasterizer's own anti-aliasing produce a properly filtered edge every time, matching what
    /// [`crate::webgl::engraving::EngravingTexture`] then uploads one-level-at-a-time instead of
    /// relying on `generateMipmap`.
    ///
    /// Returns levels ready for [`crate::webgl::engraving::EngravingTexture::update`].
    pub async fn render_text_mip_chain
    (
      &mut self,
      text : &str,
      font_family : &str,
      layout : &EngravingTextLayout,
    )
    -> Result< Vec< EngravingBitmap >, EngravingCanvasError >
    {
      let char_count = text.chars().count();
      if char_count > layout.max_characters
      {
        return Err( EngravingCanvasError::TooManyCharacters { got : char_count, max : layout.max_characters } );
      }

      self.ensure_font_loaded( font_family, text ).await?;

      let mut bitmaps = Vec::new();
      let ( mut width, mut height ) = ( self.width, self.height );
      let mut is_base_level = true;
      loop
      {
        bitmaps.push( self.render_at_size( text, font_family, layout, width, height, is_base_level )? );
        if width == 1 && height == 1
        {
          break;
        }
        width = ( width / 2 ).max( 1 );
        height = ( height / 2 ).max( 1 );
        is_base_level = false;
      }

      // Leave the exposed debug-preview canvas at the base size rather than whatever the
      // smallest mip level happened to be.
      self.canvas.set_width( self.width );
      self.canvas.set_height( self.height );

      Ok( bitmaps )
    }

    /// Rasterizes `text` in a `width` x `height` canvas: white glyphs on a solid black
    /// background, sized and aligned per `layout`'s resolved [`SizingMode`] (see
    /// [`Self::resolve_font_size`]) within `padding` (a fraction of the shorter dimension) of
    /// every edge. Resizes (and thus clears) `self.canvas` as a side effect.
    ///
    /// `is_base_level` gates whether `PHYSICAL`/`HYBRID` overflow is fatal — see
    /// [`Self::resolve_font_size`].
    fn render_at_size
    (
      &mut self,
      text : &str,
      font_family : &str,
      layout : &EngravingTextLayout,
      width : u32,
      height : u32,
      is_base_level : bool,
    )
    -> Result< EngravingBitmap, EngravingCanvasError >
    {
      self.canvas.set_width( width );
      self.canvas.set_height( height );

      // Fully opaque black background.
      self.ctx.set_fill_style_str( "#000000" );
      self.ctx.fill_rect( 0.0, 0.0, width as f64, height as f64 );

      if text.is_empty()
      {
        return self.read_back( width, height );
      }

      let pad_px = layout.padding * width.min( height ) as f32;
      let max_text_width = ( width as f32 - 2.0 * pad_px ).max( 1.0 );
      let max_text_height = ( height as f32 - 2.0 * pad_px ).max( 1.0 );

      let font_size = self.resolve_font_size( text, font_family, layout, height as f32, max_text_width, max_text_height, is_base_level )?;

      self.ctx.set_fill_style_str( "#FFFFFF" );
      self.ctx.set_font( &format!( "{font_size}px \"{font_family}\"" ) );
      let align = layout.align;
      self.ctx.set_text_align( align.css() );
      self.ctx.set_text_baseline( "middle" );

      let x = match align
      {
        TextAlign::Left => pad_px as f64,
        TextAlign::Center => width as f64 * 0.5,
        TextAlign::Right => ( width as f32 - pad_px ) as f64,
      };
      self.ctx.fill_text( text, x, height as f64 * 0.5 )?;

      self.read_back( width, height )
    }

    /// Picks the font size (CSS px) to render `text` at, following `layout`'s resolved
    /// [`SizingMode`]:
    ///
    /// - `RELATIVE` uses `fontSizeRatio * canvas_height` if set, otherwise falls back to
    ///   [`Self::fit_font_size`]'s auto-fit-to-bounds search — the original, unscaled behavior.
    /// - `PHYSICAL` / `HYBRID` derive a target size from `defaultFontSizeMm / stripHeightMm`
    ///   scaled by `canvas_height` (both fields are required present for these two modes by
    ///   [`crate::webgl::engraving::EngravingNodeConfig::validate`] — but that only runs at
    ///   config-load time, and [`crate::webgl::engraving::EngravingSession::text_layout_mut`]
    ///   allows mutating a node's layout afterwards, e.g. from a live GUI; if either field is
    ///   missing here, this falls back to the same auto-fit as `RELATIVE` rather than
    ///   panicking). `PHYSICAL` rejects overflow of `max_width` outright; `HYBRID` shrinks
    ///   proportionally down to `minFontSizeMm` (unboundedly if unset) before rejecting.
    ///
    /// `is_base_level` gates whether that rejection is actually fatal: it only is for the
    /// full-resolution mip level. A mip chain's lower levels exist purely to anti-alias the
    /// minified result, not to independently satisfy the physical-size contract — and at a few
    /// pixels of canvas, `measure_text` width no longer scales linearly with font size (glyph
    /// hinting/rounding), so a real, correctly-sized base level can still "overflow" a tiny mip
    /// through no fault of the config. Rejecting there would abort the entire chain (including
    /// the already-fine base level) for a level nobody looks at directly, so non-base levels
    /// fall back to a best-effort fit instead of erroring.
    fn resolve_font_size
    (
      &self,
      text : &str,
      font_family : &str,
      layout : &EngravingTextLayout,
      canvas_height : f32,
      max_width : f32,
      max_height : f32,
      is_base_level : bool,
    )
    -> Result< f32, EngravingCanvasError >
    {
      match layout.resolved_sizing_mode()
      {
        SizingMode::Relative =>
        {
          if let Some( ratio ) = layout.font_size_ratio
          {
            return Ok( ( canvas_height * ratio ).max( 1.0 ) );
          }
          self.fit_font_size( text, font_family, max_width, max_height )
        }
        mode @ ( SizingMode::Physical | SizingMode::Hybrid ) =>
        {
          let ( Some( strip_height_mm ), Some( target_mm ) ) = ( layout.strip_height_mm, layout.default_font_size_mm )
          else
          {
            return self.fit_font_size( text, font_family, max_width, max_height );
          };
          let target_px = ( canvas_height * ( target_mm / strip_height_mm ) ).max( 1.0 );

          let ( target_w, _ ) = self.measure( text, font_family, target_px )?;
          if target_w <= max_width
          {
            return Ok( target_px );
          }

          if mode == SizingMode::Physical
          {
            if !is_base_level
            {
              return self.fit_font_size( text, font_family, max_width, max_height );
            }
            return Err( EngravingCanvasError::TextOverflow { text_width_px : target_w, max_width_px : max_width } );
          }

          let shrunk_px = target_px * ( max_width / target_w );
          let floor_px = if is_base_level
          {
            layout.min_font_size_mm.map( | min_mm | canvas_height * ( min_mm / strip_height_mm ) ).unwrap_or( 1.0 )
          }
          else
          {
            // Non-base levels never reject overflow (see the doc comment above), so there is no
            // floor to hold — always shrink all the way down to whatever fits.
            1.0
          };
          let final_px = shrunk_px.max( floor_px );

          // Only re-measure when the floor actually raised the size back above the fit-to-width
          // estimate — otherwise `final_px == shrunk_px`, which fits `max_width` by construction.
          if final_px > shrunk_px && is_base_level
          {
            let ( final_w, _ ) = self.measure( text, font_family, final_px )?;
            if final_w > max_width
            {
              return Err( EngravingCanvasError::TextOverflow { text_width_px : final_w, max_width_px : max_width } );
            }
          }

          Ok( final_px )
        }
      }
    }

    /// Sets `self.ctx`'s font to `size`px `font_family` and measures `text` against it: `( width, ink-height )`.
    /// Ink height is `actualBoundingBoxAscent + actualBoundingBoxDescent` rather than the font's
    /// nominal em-height, since engraving strips are shallow and an em-box size would under-fill
    /// them for text without descenders (e.g. all-caps).
    fn measure( &self, text : &str, font_family : &str, size : f32 ) -> Result< ( f32, f32 ), EngravingCanvasError >
    {
      self.ctx.set_font( &format!( "{size}px \"{font_family}\"" ) );
      let metrics = self.ctx.measure_text( text )?;
      let w = metrics.width() as f32;
      let h = ( metrics.actual_bounding_box_ascent() + metrics.actual_bounding_box_descent() ) as f32;
      Ok( ( w, h ) )
    }

    /// Binary-searches the largest font size (in CSS px) for which `text` set in `font_family`
    /// fits inside a `max_width` x `max_height` box.
    fn fit_font_size( &self, text : &str, font_family : &str, max_width : f32, max_height : f32 ) -> Result< f32, EngravingCanvasError >
    {
      let fits = | size : f32 | -> Result< bool, EngravingCanvasError >
      {
        let ( w, h ) = self.measure( text, font_family, size )?;
        Ok( w <= max_width && h <= max_height )
      };

      let mut lo = 1.0_f32;
      let mut hi = max_height;

      // `lo` is only advanced once proven to fit, so it is always a valid answer, even if the
      // loop below never runs (e.g. max_width/max_height are already smaller than 1px fits).
      if !fits( lo )?
      {
        return Ok( lo );
      }

      for _ in 0..24
      {
        let mid = 0.5 * ( lo + hi );
        if fits( mid )?
        {
          lo = mid;
        }
        else
        {
          hi = mid;
        }
        if hi - lo < 0.25
        {
          break;
        }
      }

      Ok( lo )
    }

    fn read_back( &self, width : u32, height : u32 ) -> Result< EngravingBitmap, EngravingCanvasError >
    {
      // The workspace builds with `--cfg web_sys_unstable_apis` (see .cargo/config.toml), which
      // switches `get_image_data` to the i32-coordinate overload.
      let image_data = self.ctx.get_image_data( 0, 0, width as i32, height as i32 )?;
      Ok( EngravingBitmap { width, height, pixels : image_data.data().to_vec() } )
    }
  }

}

crate::mod_interface!
{
  orphan use
  {
    EngravingCanvasManager,
    EngravingBitmap,
    EngravingCanvasError,
  };
}
