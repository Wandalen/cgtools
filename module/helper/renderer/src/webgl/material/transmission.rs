mod private
{
  /// Distance the refracted ray is carried inside the object, for a body of
  /// `thickness` world units. Both the screen-space offset of the refracted tap
  /// and the Beer–Lambert absorption path use it.
  ///
  /// It is simply the thickness, with no dependence on the angle of the interior
  /// ray — the three.js / glTF-sample-viewer model, which offsets by
  /// `refract( -v, n, 1/ior ) * thickness * modelScale`. The reason is not that
  /// the angle does not matter but that a screen-space capture cannot pay for
  /// caring. Two angle-dependent readings were implemented and measured first:
  ///
  /// 1. **Flat slab, `thickness / |cos|`.** Exact for a parallel-sided pane, but
  ///    the cosine is on the wrong side for anything convex: it lengthens the
  ///    path towards the silhouette where the real chord shortens. In practice it
  ///    folded the rim across to the far side of the object and smeared it.
  ///    A genuine pane is the thin-walled case anyway, which never reaches here.
  /// 2. **Convex chord, `thickness * |cos|`.** Exact for a sphere of diameter
  ///    `thickness`, and with a second refraction at the exit it reproduced an
  ///    analytic ball lens to float precision. It still looked wrong, because a
  ///    real ball is a *wide-angle* instrument: traced at IOR 1.5, a pixel 80% of
  ///    the way to the silhouette reads the scene 1.6 radii off-axis and one at
  ///    98% reads 7.7 radii off. All of that is outside the frame, so the outer
  ///    band of every transmissive object fell back to the un-refracted pixel —
  ///    i.e. to whatever sits directly behind it — and the band a crystal ball is
  ///    actually recognised by is the part a screen-space capture cannot supply.
  ///
  /// So the angle is dropped on purpose. A constant offset keeps every tap near
  /// the object and on-screen, degrades gracefully instead of falling off a cliff
  /// at the silhouette, and matches what other real-time renderers produce for
  /// the same asset. Recovering the faithful behaviour needs the captured scene
  /// to extend past the frame — a cube-map or a wider capture — not a better
  /// formula here.
  ///
  /// The same expression is evaluated by `main.frag`'s `USE_TRANSMISSION` block,
  /// which is why it lives here as plain, natively-testable arithmetic rather
  /// than only in GLSL.
  #[ must_use ]
  pub fn traversal_length( thickness : f32 ) -> f32
  {
    if thickness.is_finite() { thickness.max( 0.0 ) } else { 0.0 }
  }

  /// Per-channel transmittance of the interior medium over `path_length` world
  /// units, from the OpenPBR volumetric-absorption parameters.
  ///
  /// The spec defines the extinction coefficient of the interior medium by
  /// Beer's law as
  ///
  /// ```text
  /// mu_t = -ln( T ) / lambda
  /// ```
  ///
  /// where `T` is `transmission_color` — the color white light becomes after
  /// travelling `lambda` = `transmission_depth` through the medium — so the
  /// transmittance over a distance `d` is `exp( -mu_t * d )`, i.e. `T^( d / lambda )`.
  /// ( The glTF `KHR_materials_volume` carriers spell the same relation as
  /// `attenuationColor ^ ( d / attenuationDistance )`, so both ingestion lanes
  /// land on this one function. )
  ///
  /// `transmission_depth <= 0` ( or non-finite, the glTF `+inf` schema default
  /// for `attenuationDistance` ) is the spec's "the interior medium is absent"
  /// case: `transmission_color` is then returned unchanged, tinting the
  /// refraction by a constant amount — non-physical, but what the spec
  /// prescribes for `lambda = 0`.
  #[ must_use ]
  pub fn transmittance( transmission_color : [ f32; 3 ], transmission_depth : f32, path_length : f32 ) -> [ f32; 3 ]
  {
    if transmission_depth <= 0.0 || !transmission_depth.is_finite()
    {
      return transmission_color;
    }

    let d = path_length.max( 0.0 );
    let mut out = [ 0.0_f32; 3 ];
    for ( o, c ) in out.iter_mut().zip( transmission_color.iter() )
    {
      // `c = 0` means "opaque at one depth unit": the log diverges, so the
      // channel is clamped to a very small but finite color instead of
      // producing -inf * 0 = NaN at `path_length = 0`.
      let color = c.clamp( 1e-5, 1.0 );
      let mu_t = -color.ln() / transmission_depth;
      *o = ( -mu_t * d ).exp();
    }
    out
  }

  /// Mip level of the captured transmission target that approximates the
  /// roughness-driven blur of a refracted ray.
  ///
  /// Screen-space refraction takes a single tap, so surface roughness has to be
  /// emulated by pre-filtering: the transmission target carries a full mip
  /// chain and a rougher surface reads from a coarser level. The IOR factor
  /// ( `clamp( 2 * ior - 2, 0, 1 )` ) follows the three.js `applyIorToRoughness`
  /// heuristic — the refracted lobe widens with the index contrast, so a
  /// nearly-IOR-1 medium stays sharp however rough its surface is.
  ///
  /// `target_size` is the transmission target's larger dimension in pixels; the
  /// result is clamped to the chain's own top level, `log2( target_size )`.
  #[ must_use ]
  pub fn blur_lod( roughness : f32, ior : f32, target_size : f32 ) -> f32
  {
    let max_lod = target_size.max( 1.0 ).log2();
    let ior_factor = ( 2.0 * ior - 2.0 ).clamp( 0.0, 1.0 );
    ( max_lod * ( roughness.clamp( 0.0, 1.0 ) * ior_factor ) ).clamp( 0.0, max_lod )
  }

  /// Widest radius, in texels, the occluder rejection of a refracted tap is
  /// dilated by. Beyond a few texels the rejected band starts eating the
  /// refraction itself, so a very blurred tap keeps some fringe rather than
  /// losing the image - screen-space transmission cannot win that trade.
  pub const OCCLUDER_DILATION_MAX_TEXELS : f32 = 4.0;

  /// Radius, in texels, over which the refracted tap’s occluder test looks for
  /// a nearer surface, given the mip level the color is read at.
  ///
  /// The test needs a radius at all because the color and the depth of the
  /// capture do not agree at a silhouette. The color is read with filtering -
  /// bilinearly at level 0, over `2^lod` texels above it, and averaged across
  /// MSAA samples when the capture resolved one - while the depth is a single
  /// nearest texel. A tap sitting just outside a foreground object therefore
  /// carries some of that object’s color under a background depth, passes the
  /// test, and paints a bright fringe tracing every foreground edge seen
  /// through the glass. Rejecting the tap when a *neighbourhood* of the depth
  /// contains an occluder drops that contaminated band instead.
  ///
  /// One texel is the floor ( the bilinear footprint is always there ), the
  /// mip footprint `2^lod` grows it, and [`OCCLUDER_DILATION_MAX_TEXELS`] caps
  /// it. A non-finite `blur_lod` falls back to the floor.
  #[ must_use ]
  pub fn occluder_dilation_texels( blur_lod : f32 ) -> f32
  {
    if !blur_lod.is_finite()
    {
      return 1.0;
    }
    blur_lod.max( 0.0 ).exp2().clamp( 1.0, OCCLUDER_DILATION_MAX_TEXELS )
  }

  /// Geometric slab thickness to refract through when the material authors
  /// none, derived from the world-space bounding-box `extent` of the primitive
  /// being drawn.
  ///
  /// OpenPBR has no geometric-thickness parameter at all - `transmission_depth`
  /// is an absorption length scale, not a distance through the object - and a
  /// `.mtlx` / `.usda` surface therefore arrives with nothing to offset the
  /// refracted ray by. Two answers are defensible, and the renderer takes the
  /// first:
  ///
  /// 1. **Object-sized slab ( what this is ).** Fall back to the size of the
  ///    object itself, so the refraction is scale-invariant : the same glass
  ///    sphere refracts the same way whether the scene is authored in metres or
  ///    normalised to a unit box. A fixed world-space constant cannot be right
  ///    for both.
  /// 2. **Thin-walled.** glTF says a material without `KHR_materials_volume` has
  ///    no volume, so the strictly spec-faithful reading is no offset and no
  ///    absorption at all ( `USE_TRANSMISSION_THIN_WALLED` ). Correct by the
  ///    letter of both specs, but an unauthored glass ball then stops refracting
  ///    entirely. Switching to it means returning `0.0` here unconditionally -
  ///    the degenerate-extent path below already behaves that way.
  ///
  /// The *smallest* extent is taken, because it is the shortest way through the
  /// object : right for a pane or a slab, and for a sphere it is the diameter,
  /// which over-states the average chord but keeps the screen-space offset
  /// modest. Non-finite or non-positive extents are ignored; when none survives
  /// the result is `0.0`, which collapses the slab to the thin-walled case
  /// rather than producing a garbage offset.
  #[ must_use ]
  pub fn fallback_slab_thickness( extent : [ f32; 3 ] ) -> f32
  {
    let smallest = extent
    .iter()
    .copied()
    .filter( | e | e.is_finite() && *e > 0.0 )
    .fold( f32::INFINITY, f32::min );

    if smallest.is_finite() { smallest } else { 0.0 }
  }
}

crate::mod_interface!
{
  orphan use
  {
    traversal_length,
    transmittance,
    blur_lod,
    OCCLUDER_DILATION_MAX_TEXELS,
    occluder_dilation_texels,
    fallback_slab_thickness
  };
}
