//! Verifies the pure transmission math shared by `main.frag`'s
//! `USE_TRANSMISSION` block and `renderer::webgl::material` ( `transmission.rs` ),
//! plus the surface-to-carrier mapping that feeds it.
//!
//! The shader itself cannot be unit-tested ( GLSL ES 3.00 only compiles inside
//! a browser - see `shader_validation_tests.rs` for why naga is not an option
//! here ), so the arithmetic it evaluates is lifted into Rust, tested against
//! the ASWF OpenPBR Surface spec's own equations, and mirrored line-for-line in
//! the shader. Zero `WebGl2RenderingContext` calls: the same off-GPU pattern as
//! `gltf_material_extensions_test.rs`.
//!
//! Spec reference ( <https://academysoftwarefoundation.github.io/OpenPBR/> ):
//! `transmission_depth` is the distance white light travels inside the medium
//! before its color becomes exactly `transmission_color` by Beer's law,
//! `mu_t = -ln( T ) / lambda`; at `lambda = 0` the interior medium is absent and
//! `transmission_color` instead tints the refraction by a constant amount.

#![ cfg( not( target_arch = "wasm32" ) ) ]

use renderer::webgl::material::
{
  OpenPbrSurface,
  OCCLUDER_DILATION_MAX_TEXELS,
  blur_lod,
  fallback_slab_thickness,
  occluder_dilation_texels,
  openpbr_params_from_surface,
  traversal_length,
  transmittance,
};

/// Asserts per-channel equality within a tolerance, reporting both triples.
fn assert_rgb_close( actual : [ f32; 3 ], expected : [ f32; 3 ], tolerance : f32 )
{
  for ( i, ( a, e ) ) in actual.iter().zip( expected.iter() ).enumerate()
  {
    assert!
    (
      ( a - e ).abs() <= tolerance,
      "channel {i}: {actual:?} != {expected:?} ( tolerance {tolerance} )"
    );
  }
}

/// The traversal is the thickness, flat: no angle dependence, so the refracted
/// tap never travels further just because the ray is closer to the silhouette.
/// That is the whole point of the constant model - the angle-dependent readings
/// throw the outer band of the object off-screen.
#[ test ]
fn traversal_is_the_thickness()
{
  assert!( ( traversal_length( 0.5 ) - 0.5 ).abs() < 1e-6 );
  assert!( ( traversal_length( 2.0 ) - 2.0 ).abs() < 1e-6 );
  // A negative thickness is meaningless; it must not yield a negative path.
  assert!( traversal_length( -1.0 ).abs() < 1e-6 );
}

/// A degenerate thickness must not become a `NaN` offset, which would take the
/// whole fragment with it.
#[ test ]
fn traversal_survives_non_finite_thickness()
{
  for bad in [ f32::NAN, f32::INFINITY, f32::NEG_INFINITY ]
  {
    let d = traversal_length( bad );
    assert!( d.is_finite() && d.abs() < 1e-9, "thickness {bad} gave {d}" );
  }
}

/// Beer's law, stated as the spec does: after exactly one `transmission_depth`
/// the medium turns white light into `transmission_color`.
#[ test ]
fn transmittance_reaches_transmission_color_at_one_depth()
{
  let color = [ 0.9, 0.5, 0.2 ];
  assert_rgb_close( transmittance( color, 2.0, 2.0 ), color, 1e-5 );
  // Half the distance: sqrt of the color ( T^( d / lambda ) ).
  assert_rgb_close
  (
    transmittance( color, 2.0, 1.0 ),
    [ color[ 0 ].sqrt(), color[ 1 ].sqrt(), color[ 2 ].sqrt() ],
    1e-5
  );
  // Twice the distance: the color squared.
  assert_rgb_close
  (
    transmittance( color, 2.0, 4.0 ),
    [ color[ 0 ] * color[ 0 ], color[ 1 ] * color[ 1 ], color[ 2 ] * color[ 2 ] ],
    1e-5
  );
  // No distance travelled -> nothing absorbed.
  assert_rgb_close( transmittance( color, 2.0, 0.0 ), [ 1.0, 1.0, 1.0 ], 1e-5 );
}

/// A white medium never absorbs, however far the ray travels.
#[ test ]
fn transmittance_of_white_is_transparent_at_any_distance()
{
  assert_rgb_close( transmittance( [ 1.0, 1.0, 1.0 ], 0.25, 100.0 ), [ 1.0, 1.0, 1.0 ], 1e-5 );
}

/// `transmission_depth = 0` is the spec's "interior medium is absent" case: the
/// color becomes a constant tint rather than a volumetric absorption. The glTF
/// `attenuationDistance` schema default ( `+inf`, carried as a non-finite value )
/// means the same thing - a non-absorbing medium.
#[ test ]
fn transmittance_without_a_depth_degenerates_to_a_constant_tint()
{
  let color = [ 0.9, 0.5, 0.2 ];
  assert_rgb_close( transmittance( color, 0.0, 3.0 ), color, 0.0 );
  assert_rgb_close( transmittance( color, -1.0, 3.0 ), color, 0.0 );
  assert_rgb_close( transmittance( color, f32::INFINITY, 3.0 ), color, 0.0 );
}

/// A fully absorbing channel must stay finite ( the `-ln( 0 )` pole is clamped ),
/// and still let everything through at zero distance.
#[ test ]
fn transmittance_handles_a_fully_absorbing_channel()
{
  let out = transmittance( [ 0.0, 0.5, 1.0 ], 1.0, 1.0 );
  assert!( out.iter().all( | c | c.is_finite() ), "{out:?}" );
  assert!( out[ 0 ] < 1e-4, "an opaque channel absorbs nearly everything: {out:?}" );
  assert_rgb_close( transmittance( [ 0.0, 0.5, 1.0 ], 1.0, 0.0 ), [ 1.0, 1.0, 1.0 ], 1e-5 );
}

/// The blur level grows with roughness and with the index contrast, and never
/// leaves the mip chain that actually exists.
#[ test ]
fn blur_lod_scales_with_roughness_and_ior()
{
  // 1024 px target -> 10 levels above the base.
  assert!( ( blur_lod( 0.0, 1.5, 1024.0 ) - 0.0 ).abs() < 1e-6 );
  // ior 1.0 means no index contrast: nothing refracts, nothing blurs.
  assert!( ( blur_lod( 1.0, 1.0, 1024.0 ) - 0.0 ).abs() < 1e-6 );
  // ior 1.5 -> factor clamp( 2 * 1.5 - 2, 0, 1 ) = 1 -> the full chain.
  assert!( ( blur_lod( 1.0, 1.5, 1024.0 ) - 10.0 ).abs() < 1e-6 );
  assert!( ( blur_lod( 0.5, 1.5, 1024.0 ) - 5.0 ).abs() < 1e-6 );
  // Monotonic in roughness, and clamped into the chain at any input.
  assert!( blur_lod( 0.3, 1.5, 1024.0 ) < blur_lod( 0.6, 1.5, 1024.0 ) );
  assert!( blur_lod( 5.0, 9.0, 1024.0 ) <= 10.0 );
  assert!( blur_lod( -1.0, 1.5, 1024.0 ) >= 0.0 );
}

/// `transmission_depth` is an absorption length scale, so it must ride the glTF
/// `attenuationDistance` carrier - not `thicknessFactor`, which is the geometric
/// depth of the slab. Mixing them up would drive the refraction offset with an
/// absorption distance ( and leave the absorption unfed ).
#[ test ]
fn params_from_surface_maps_transmission_depth_to_the_attenuation_distance()
{
  let mut surface = OpenPbrSurface::spec_default();
  surface.transmission_weight = 1.0;
  surface.transmission_depth = 0.4;
  surface.transmission_color = [ 0.9, 0.95, 1.0 ];

  let params = openpbr_params_from_surface( &surface );
  assert_eq!( params.transmission_factor, Some( 1.0 ) );
  assert_eq!( params.volume_attenuation_distance, Some( 0.4 ) );
  assert_eq!( params.volume_attenuation_color, Some( [ 0.9, 0.95, 1.0 ] ) );
  // Not thin-walled and OpenPBR carries no geometric thickness of its own, so
  // the carrier stays unset and the material falls back to the shader default.
  assert_eq!( params.volume_thickness_factor, None );
}

/// `geometry_thin_walled` is the spec's zero-thickness shell, which is exactly
/// the glTF `thicknessFactor = 0` case the shader compiles its thin-walled
/// variant from.
#[ test ]
fn params_from_surface_maps_thin_walled_to_zero_thickness()
{
  let mut surface = OpenPbrSurface::spec_default();
  surface.transmission_weight = 1.0;
  surface.geometry_thin_walled = true;

  let params = openpbr_params_from_surface( &surface );
  assert_eq!( params.volume_thickness_factor, Some( 0.0 ) );

  // An opaque thin-walled surface transmits nothing, so no carrier is emitted.
  let mut opaque = OpenPbrSurface::spec_default();
  opaque.geometry_thin_walled = true;
  let params = openpbr_params_from_surface( &opaque );
  assert_eq!( params.volume_thickness_factor, None );
  assert_eq!( params.transmission_factor, None );
}

/// The bilinear footprint of the colour tap is always there, so the rejection is
/// never narrower than a texel even for a perfectly sharp refraction.
#[ test ]
fn occluder_dilation_never_falls_below_one_texel()
{
  for lod in [ 0.0_f32, -1.0, -100.0 ]
  {
    let r = occluder_dilation_texels( lod );
    assert!( ( r - 1.0 ).abs() < 1e-6, "lod {lod} gave {r}, expected the one-texel floor" );
  }
}

/// A blurrier tap gathers colour from further away, so it can be contaminated
/// from further away and the test has to look further — up to the cap, past
/// which the rejected band would start eating the refraction itself.
#[ test ]
fn occluder_dilation_follows_the_mip_footprint_up_to_the_cap()
{
  assert!( ( occluder_dilation_texels( 1.0 ) - 2.0 ).abs() < 1e-6 );
  assert!( ( occluder_dilation_texels( 2.0 ) - 4.0 ).abs() < 1e-6 );
  for lod in [ 3.0_f32, 8.0, 16.0 ]
  {
    let r = occluder_dilation_texels( lod );
    assert!
    (
      ( r - OCCLUDER_DILATION_MAX_TEXELS ).abs() < 1e-6,
      "lod {lod} gave {r}, expected the cap {OCCLUDER_DILATION_MAX_TEXELS}"
    );
  }
}

/// The radius is monotone in the blur level: a sharper tap is never tested over
/// a wider neighbourhood than a blurrier one.
#[ test ]
fn occluder_dilation_is_monotone_in_the_blur_level()
{
  let mut previous = 0.0_f32;
  for lod in [ 0.0_f32, 0.5, 1.0, 1.5, 2.0, 3.0, 6.0 ]
  {
    let r = occluder_dilation_texels( lod );
    assert!( r >= previous, "lod {lod}: {r} fell below {previous}" );
    previous = r;
  }
}

/// A degenerate blur level must not hand the shader a `NaN` radius — the whole
/// rejection would collapse and the fringe would come back.
#[ test ]
fn occluder_dilation_survives_a_non_finite_blur_level()
{
  for lod in [ f32::NAN, f32::INFINITY, f32::NEG_INFINITY ]
  {
    let r = occluder_dilation_texels( lod );
    assert!( r.is_finite(), "lod {lod} gave a non-finite radius {r}" );
    assert!( ( 1.0 ..= OCCLUDER_DILATION_MAX_TEXELS ).contains( &r ), "lod {lod} gave {r} outside the range" );
  }
}

/// The slab falls back to the *shortest* way through the object: the right
/// answer for a pane or a slab, and the one that keeps the screen-space offset
/// modest for anything chunkier.
#[ test ]
fn fallback_thickness_is_the_smallest_extent()
{
  assert!( ( fallback_slab_thickness( [ 2.0, 0.1, 3.0 ] ) - 0.1 ).abs() < 1e-6 );
  assert!( ( fallback_slab_thickness( [ 0.25, 0.25, 0.25 ] ) - 0.25 ).abs() < 1e-6 );
  assert!( ( fallback_slab_thickness( [ 5.0, 4.0, 0.001 ] ) - 0.001 ).abs() < 1e-6 );
}

/// The whole point of deriving it from the object: the same object authored at a
/// different scale must refract the same way, which a fixed world-space constant
/// cannot do.
#[ test ]
fn fallback_thickness_tracks_the_object_scale()
{
  let unit = fallback_slab_thickness( [ 1.0, 1.0, 1.0 ] );
  let tenth = fallback_slab_thickness( [ 0.1, 0.1, 0.1 ] );
  assert!( ( tenth * 10.0 - unit ).abs() < 1e-5, "{tenth} * 10 != {unit}" );
}

/// A flat primitive ( a plane, or a mesh whose bounding box was never computed )
/// has a degenerate extent along at least one axis. Those axes are ignored
/// rather than collapsing the slab, so a card still refracts through its
/// in-plane size.
#[ test ]
fn fallback_thickness_ignores_degenerate_axes()
{
  assert!( ( fallback_slab_thickness( [ 2.0, 0.0, 3.0 ] ) - 2.0 ).abs() < 1e-6 );
  assert!( ( fallback_slab_thickness( [ -1.0, 4.0, 2.0 ] ) - 2.0 ).abs() < 1e-6 );
}

/// With nothing usable left the result is zero, which collapses the slab to the
/// thin-walled case — no offset, no absorption — instead of handing the shader a
/// garbage displacement. This is also the behaviour the thin-walled alternative
/// to the object-sized slab would have everywhere.
#[ test ]
fn fallback_thickness_degrades_to_thin_walled()
{
  for extent in [ [ 0.0, 0.0, 0.0 ], [ f32::NAN; 3 ], [ f32::INFINITY; 3 ], [ -1.0, -2.0, 0.0 ] ]
  {
    let t = fallback_slab_thickness( extent );
    assert!( t.is_finite() && t.abs() < 1e-9, "extent {extent:?} gave {t}" );
  }
}

/// An inverted bounding box ( the `BoundingBox::default()` sentinel: `min = +inf`,
/// `max = -inf` ) reaches this as a `-inf` extent. It must not become a slab.
#[ test ]
fn fallback_thickness_survives_an_uncomputed_bounding_box()
{
  let t = fallback_slab_thickness( [ f32::NEG_INFINITY; 3 ] );
  assert!( t.is_finite() && t.abs() < 1e-9, "uncomputed box gave {t}" );
}
