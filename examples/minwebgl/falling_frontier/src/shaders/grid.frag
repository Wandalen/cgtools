#version 300 es
precision highp float;

// Tactical grid — ported from examples/threejs/falling_frontier's
// tacticalGrid.js fragment shader. M1 slice: fwidth-based analytic-AA grid
// lines + camera-distance fade. M3 slice (this version): the view-zone
// ribbon (a closed boundary polyline built on the CPU each frame — see
// `boundary.rs` — wrapped tight around any blocking asteroid), the
// inside/outside brightness fade, and the asteroid proximity glow.
//
// MAX_BOUNDARY_PTS/MAX_ASTEROID_GLOW below must match
// `boundary::MAX_BOUNDARY_PTS` / `main::MAX_ASTEROID_GLOW` on the Rust side —
// they size the fixed uniform arrays uploaded from there.

uniform vec3 u_line_color;
uniform vec3 u_ring_color_core;
uniform vec3 u_ring_color_edge;
uniform float u_dim_alpha;
uniform float u_bright_alpha;
uniform vec2 u_focus_point;
uniform float u_focus_active;
uniform float u_view_radius;
uniform float u_cell_size;
uniform float u_line_width_px;
uniform float u_ribbon_width_outer;
uniform float u_ribbon_width_inner;
uniform float u_ribbon_gap;
uniform float u_ribbon_opacity;
uniform float u_inside_fade_width;
uniform float u_inside_fade_mode;
uniform float u_inside_fade_gamma;
uniform float u_camera_fade_start;
uniform float u_camera_fade_end;
uniform float u_camera_fade_mode;
uniform float u_camera_fade_gamma;
uniform vec2 u_boundary_pts[ 128 ];
uniform int u_boundary_count;
uniform vec2 u_asteroid_pos[ 16 ];
uniform float u_asteroid_radius[ 16 ];
uniform int u_asteroid_count;
uniform float u_asteroid_glow_alpha;
uniform float u_asteroid_glow_width;
uniform float u_asteroid_glow_mode;
uniform float u_asteroid_glow_gamma;
uniform vec3 u_camera_position;

in vec3 v_world_pos;

layout( location = 0 ) out vec4 frag_color;

// Euclidean distance from point p to the segment [a, b].
float distToSegment( vec2 p, vec2 a, vec2 b )
{
  vec2 ab = b - a;
  float t = clamp( dot( p - a, ab ) / max( dot( ab, ab ), 0.0001 ), 0.0, 1.0 );
  return length( p - ( a + ab * t ) );
}

// smoothstep is undefined ( can return NaN ) when edge0 == edge1 -- not just for literally
// equal inputs, but also when a large edge0/edge1 magnitude absorbs a small constant offset
// in floating-point ( e.g. halfWidth - 0.6 == halfWidth once halfWidth is large enough that
// 0.6 falls below f32's representable precision at that magnitude ). Falls back to a hard
// step at edge0, matching smoothstep's own limit as edge1 approaches edge0 from above.
float safeSmoothstep( float edge0, float edge1, float x )
{
  if ( edge0 == edge1 )
  {
    return x < edge0 ? 0.0 : 1.0;
  }
  return smoothstep( edge0, edge1, x );
}

// 1 where |signedDist - target| < halfWidth, antialiased falloff outside.
float ribbonMask( float signedDist, float target, float halfWidth )
{
  return 1.0 - safeSmoothstep( halfWidth - 0.6, halfWidth, abs( signedDist - target ) );
}

// Returns 1 for x <= start, 0 for x >= end (exponential modes are
// asymptotic so never quite hit exactly 0), shaped by "mode":
//   0 = linear, 1 = smoothstep (ease), 2 = exponential, 3 = exponential^2
// gamma reshapes the result afterwards (< 1 fades earlier/faster,
// > 1 holds near 1 longer then drops off quicker at the end).
float fadeFactor( float x, float start, float end, float mode, float gamma )
{
  float span = max( end - start, 0.0001 );
  float t;
  if ( mode < 0.5 )
  {
    t = clamp( ( x - start ) / span, 0.0, 1.0 );
  }
  else if ( mode < 1.5 )
  {
    t = safeSmoothstep( start, end, x );
  }
  else
  {
    float k = 3.0 / span;
    float d = max( x - start, 0.0 );
    if ( mode < 2.5 )
    {
      t = 1.0 - exp( -k * d );
    }
    else
    {
      float e = k * d;
      t = 1.0 - exp( -e * e );
    }
  }
  return pow( clamp( 1.0 - t, 0.0, 1.0 ), max( gamma, 0.001 ) );
}

void main()
{
  // Anti-aliased grid lines every u_cell_size world units. fwidth(coord)
  // alone gives a line exactly 1 screen-pixel wide (the thinnest
  // representable AA line) — dividing by it directly reads as razor-thin
  // and washes out at any distance, so u_line_width_px widens the covered
  // band to an actually legible pixel width without softening the edges.
  vec2 coord = v_world_pos.xz / u_cell_size;
  vec2 gridUv = abs( fract( coord - 0.5 ) - 0.5 ) / ( fwidth( coord ) * u_line_width_px );
  float lineMask = 1.0 - min( min( gridUv.x, gridUv.y ), 1.0 );

  // Fades the whole plane out with distance from the camera, so the
  // "infinite" grid dissolves into haze instead of tiling forever.
  float camDist = length( v_world_pos - u_camera_position );
  float camFade = fadeFactor( camDist, u_camera_fade_start, u_camera_fade_end, u_camera_fade_mode, u_camera_fade_gamma );

  vec2 fragXZ = v_world_pos.xz;
  float innerHalf = u_ribbon_width_inner * 0.5;
  float outerHalf = u_ribbon_width_outer * 0.5;
  float gapHalf = u_ribbon_gap * 0.5;

  // The view-zone boundary is baked on the CPU each frame into an explicit
  // closed polyline (u_boundary_pts) - a faceted circle around the focus
  // point, replaced locally by segments that wrap the near side of any
  // blocking asteroid (see boundary.rs). Here we just need the true
  // Euclidean distance from this fragment to the nearest point on that
  // polyline (for the ribbon) and whether the fragment is inside it (for the
  // brightness fade) via a standard crossing-number test - both computed in
  // one pass over the same point list, so the ribbon is always a single
  // continuous, constant-width, constant-gap band with no separate
  // "circle vs. arc" logic to fall out of sync.
  float boundaryDist = 1.0e6;
  bool insideFlag = false;
  float inside = 0.0;
  float ringInner = 0.0;
  float ringOuter = 0.0;
  float ring = 0.0;
  float asteroidGlow = 0.0;

  if ( u_focus_active > 0.5 && u_boundary_count > 0 )
  {
    float dist = length( fragXZ - u_focus_point );
    float nearZone = u_view_radius + outerHalf + gapHalf + u_inside_fade_width * u_cell_size + 4.0;

    if ( dist < nearZone )
    {
      for ( int i = 0; i < 128; i++ )
      {
        if ( i >= u_boundary_count ) break;
        int j = ( i + 1 < u_boundary_count ) ? ( i + 1 ) : 0;
        vec2 a = u_boundary_pts[ i ];
        vec2 b = u_boundary_pts[ j ];
        boundaryDist = min( boundaryDist, distToSegment( fragXZ, a, b ) );

        if ( ( a.y > fragXZ.y ) != ( b.y > fragXZ.y ) )
        {
          float xCross = a.x + ( b.x - a.x ) * ( fragXZ.y - a.y ) / ( b.y - a.y );
          if ( fragXZ.x < xCross )
          {
            insideFlag = !insideFlag;
          }
        }
      }

      float signedDist = insideFlag ? -boundaryDist : boundaryDist;

      // 1 well inside the boundary, fading down to 0 (dim) over
      // u_inside_fade_width grid squares as it approaches the ribbon.
      float insideFadeStart = -u_inside_fade_width * u_cell_size;
      inside = u_focus_active * fadeFactor( signedDist, insideFadeStart, 0.0, u_inside_fade_mode, u_inside_fade_gamma );

      // Both ribbons are constant-offset copies of the SAME boundary
      // distance, so their width and the gap between them stay constant all
      // the way through a bend.
      ringInner = u_focus_active * ribbonMask( signedDist, -( gapHalf + innerHalf ), innerHalf );
      ringOuter = u_focus_active * ribbonMask( signedDist, gapHalf + outerHalf, outerHalf );
      ring = clamp( ringInner + ringOuter, 0.0, 1.0 );

      // Asteroids only brighten the grid where that ground is actually
      // visible from the focus point - inside the ribbon, never past it -
      // and u_asteroid_pos/count is itself already pre-filtered on the CPU
      // to asteroids within the current view range (Asteroids::glow_candidates),
      // so nothing outside the selected unit's view zone glows at all.
      if ( insideFlag )
      {
        float glowSpan = max( u_asteroid_glow_width * u_cell_size, 0.0001 );
        for ( int i = 0; i < 16; i++ )
        {
          if ( i >= u_asteroid_count ) break;
          float r = u_asteroid_radius[ i ];
          if ( r <= 0.0 ) continue;
          float surfaceDist = length( fragXZ - u_asteroid_pos[ i ] ) - r;
          if ( surfaceDist > glowSpan ) continue;
          float g = fadeFactor( surfaceDist, 0.0, glowSpan, u_asteroid_glow_mode, u_asteroid_glow_gamma );
          asteroidGlow = max( asteroidGlow, g );
        }
      }
    }
  }

  // The ribbon isn't a flat color - it has a brighter core that fades into a
  // more saturated edge color, antialiased by reusing each ribbon's own
  // distance-based falloff as the color mix factor.
  float ringCoreMix = clamp( max( ringInner, ringOuter ), 0.0, 1.0 );
  vec3 ringColor = mix( u_ring_color_edge, u_ring_color_core, ringCoreMix );

  float combinedMask = max( lineMask, ring );
  if ( combinedMask < 0.01 )
  {
    discard;
  }

  vec3 color = mix( u_line_color, ringColor, ring );

  // The ribbon is a tactical HUD marker, not part of the hazy "infinite"
  // grid - it stays at its own opacity regardless of camera distance, only
  // the grid lines fade with distance. The asteroid glow is a second,
  // independent brightness source - take whichever of the two (view-zone
  // focus, asteroid proximity) is stronger at this fragment rather than
  // stacking them additively.
  float focusBright = mix( u_dim_alpha, u_bright_alpha, inside );
  float glowBright = mix( u_dim_alpha, u_asteroid_glow_alpha, asteroidGlow );
  float gridAlpha = max( focusBright, glowBright ) * lineMask * camFade;
  float ringAlpha = ring * u_ribbon_opacity;
  float alpha = max( gridAlpha, ringAlpha );
  if ( alpha < 0.003 )
  {
    discard;
  }

  frag_color = vec4( color, alpha );
}
