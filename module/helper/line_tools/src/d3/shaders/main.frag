#version 300 es
// Renders 3d line, supporting both screen space and world space units.
// Allows for anti-aliasing with alpha-to-coverage enabled.
// Has an optional color attribute for the points of the line.
precision highp float;

// #include <defines>

const float MAX_FLOAT = 3.402823466e+38;

uniform vec3 u_color;
uniform float u_width;

#ifdef USE_DASH
  uniform float u_dash_offset;
  #ifdef USE_DASH_V1
    uniform float u_dash_pattern;
  #endif
  #ifdef USE_DASH_V2
    uniform vec2 u_dash_pattern;
  #endif
  #ifdef USE_DASH_V3
    uniform vec3 u_dash_pattern;
  #endif
  #ifdef USE_DASH_V4
    uniform vec4 u_dash_pattern;
  #endif
#endif

in vec2 vUv;
in vec3 vViewPos;
in vec3 vViewA;
in vec3 vViewB;

#ifdef USE_VERTEX_COLORS
  in vec3 vColor;
#endif

#ifdef USE_DASH
  in float vLineDistance;
  flat in float vLineDistanceA;
  flat in float vLineDistanceB;
#endif


out vec4 frag_color;

vec2 closestLineToLine( vec3 p1, vec3 p2, vec3 p3, vec3 p4 ) 
{

  float mua;
  float mub;

  vec3 p13 = p1 - p3;
  vec3 p43 = p4 - p3;

  vec3 p21 = p2 - p1;

  float d1343 = dot( p13, p43 );
  float d4321 = dot( p43, p21 );
  float d1321 = dot( p13, p21 );
  float d4343 = dot( p43, p43 );
  float d2121 = dot( p21, p21 );

  float denom = d2121 * d4343 - d4321 * d4321;

  float numer = d1343 * d4321 - d1321 * d4343;

  mua = numer / denom;
  mua = clamp( mua, 0.0, 1.0 );
  mub = ( d1343 + d4321 * ( mua ) ) / d4343;
  mub = clamp( mub, 0.0, 1.0 );

  return vec2( mua, mub );

}

 #ifdef USE_DASH
  float getDistanceToDash( vec3 rayEnd, float i )
  {
    #if !defined( USE_DASH_V3 ) && !defined( USE_DASH_V4 )
      #ifdef USE_DASH_V1
        float dashSize = u_dash_pattern;
        float dashGap = u_dash_pattern;
      #endif 
      
      #ifdef USE_DASH_V2
        float dashSize = u_dash_pattern.x;
        float dashGap = u_dash_pattern.y;
      #endif 

      float totalSegmentSize = dashSize + dashGap;  

      float dashCoverage = mod( vLineDistance + u_dash_offset, totalSegmentSize );

      float distanceA = vLineDistance - dashCoverage;
      float distanceB = distanceA + dashSize;

      distanceA += i * ( totalSegmentSize );
      distanceB += i * ( totalSegmentSize );
    #elif defined( USE_DASH_V3 )
      float dashSize1 = u_dash_pattern.x;
      float dashGap1 = u_dash_pattern.y;
      float dashSize2 = u_dash_pattern.z;

      float totalSegmentSize = dashSize1 + dashGap1 + dashSize2;  

      float dashCoverage = mod( vLineDistance + u_dash_offset, totalSegmentSize );

      float distanceA = 0.0;
      float distanceB = 0.0;

      if( int( floor( ( vLineDistance + u_dash_offset ) / totalSegmentSize ) ) % 2 == 0 )
      {
        float k = floor( dashCoverage / ( dashSize1 + dashGap1 ) );
        distanceA = vLineDistance - dashCoverage + mix( 0.0, dashSize1 + dashGap1, k );
        distanceB = distanceA + mix( dashSize1, dashSize2, k );

        distanceA += i * mix
        ( 
          mix( dashSize2 + dashGap1, dashSize1 + dashGap1, step( 0.0, i ) ), 
          mix( dashSize1 + dashGap1, dashSize2 + dashSize1, step( 0.0, i ) ), 
          k 
        ); 

        distanceB += i * mix
        ( 
          mix( dashSize1 + dashSize2, dashSize2 + dashGap1, step( 0.0, i ) ), 
          mix( dashSize2 + dashGap1, dashSize1 + dashGap1, step( 0.0, i ) ), 
          k 
        );
      }
      else
      {
        distanceA = vLineDistance - dashCoverage + dashSize1;
        distanceB = distanceA + dashGap1;

        distanceA += i * mix( dashSize1 + dashSize2, dashGap1 + dashSize2, step( 0.0, i ) );
        distanceB += i * mix( dashSize1 + dashGap1, dashSize2 + dashSize1, step( 0.0, i ) );
      }

    #elif defined( USE_DASH_V4 )
      float dashSize1 = u_dash_pattern.x;
      float dashGap1 = u_dash_pattern.y;
      float dashSize2 = u_dash_pattern.z;
      float dashGap2 = u_dash_pattern.w;

      float totalSegmentSize = dashSize1 + dashGap1 + dashSize2 + dashGap2;  

      float dashCoverage = mod( vLineDistance + u_dash_offset, totalSegmentSize );
      float k = min( floor( dashCoverage / ( dashSize1 + dashGap1 ) ), 1.0 );

      float distanceA = mix( vLineDistance - dashCoverage, vLineDistance - dashCoverage + dashSize1 + dashGap1, k );
      float distanceB = distanceA + mix( dashSize1, dashSize2, k );

      distanceA += i * mix
      ( 
        mix( dashSize2 + dashGap2, dashSize1 + dashGap1, step( 0.0, i ) ), 
        mix( dashSize1 + dashGap1, dashSize2 + dashGap2, step( 0.0, i ) ), 
        k 
      ); 

      distanceB += i * mix
      ( 
        mix( dashSize1 + dashGap2, dashSize2 + dashGap1, step( 0.0, i ) ), 
        mix( dashSize2 + dashGap1, dashSize1 + dashGap2, step( 0.0, i ) ), 
        k 
      ); 
    #endif

    if( distanceB <= vLineDistanceA + 1e-6 || distanceA >= vLineDistanceB - 1e-6 ) { return MAX_FLOAT; }

    vec3 lineStart = mix( vViewA, vViewB, clamp( ( distanceA - vLineDistanceA ) / ( vLineDistanceB - vLineDistanceA ), 0.0, 1.0 ) );
    vec3 lineEnd = mix( vViewA, vViewB, clamp( ( distanceB - vLineDistanceA ) / ( vLineDistanceB - vLineDistanceA ), 0.0, 1.0 ) );

    vec3 lineDir = lineEnd - lineStart;
    vec2 params = closestLineToLine( lineStart, lineEnd, vec3( 0.0, 0.0, 0.0 ), rayEnd );

    vec3 p1 = lineStart + lineDir * params.x;
    vec3 p2 = rayEnd * params.y;
    vec3 delta = p1 - p2;
    float len = length( delta );
    float norm = len / u_width;
    return norm;
  }
 #endif

void main()
{
  float alpha = 1.0;
  vec3 col = u_color;

  #ifdef USE_WORLD_UNITS
    #ifdef USE_DASH
      vec3 rayEnd = normalize( vViewPos ) * 1e5;
      
      float norm1 = getDistanceToDash( rayEnd, -1.0 );
      float norm2 = getDistanceToDash( rayEnd, 0.0 );
      float norm3 = getDistanceToDash( rayEnd, 1.0 );

      float norm = min( min( norm1, norm2 ), norm3 );
      //float norm = norm2;
      if( norm == MAX_FLOAT ) { discard; }


      #ifdef USE_ALPHA_TO_COVERAGE
        float dnorm = fwidth( norm );
        alpha = 1.0 - smoothstep( 0.5 - dnorm, 0.5 + dnorm, norm );
      #else
        if ( norm > 0.5 ) { discard; }
      #endif
    #else
      vec3 rayEnd = normalize( vViewPos ) * 1e5;
      vec3 lineDir = vViewB - vViewA;

      vec2 params = closestLineToLine( vViewA, vViewB, vec3( 0.0, 0.0, 0.0 ), rayEnd );

      vec3 p1 = vViewA + lineDir * params.x;
      vec3 p2 = rayEnd * params.y;
      vec3 delta = p1 - p2;
      float len = length( delta );
      float norm = len / u_width;


      #ifdef USE_ALPHA_TO_COVERAGE
        float dnorm = fwidth( norm );
        alpha = 1.0 - smoothstep( 0.5 - dnorm, 0.5 + dnorm, norm );
      #else
        if ( norm > 0.5 ) { discard; }
      #endif
    #endif
  #else // Screen space units
    #ifdef USE_ALPHA_TO_COVERAGE
      float a = vUv.x;
      float b = ( vUv.y > 0.0 ) ? vUv.y - 1.0 : vUv.y + 1.0;
      float len2 = a * a + b * b;
      float dlen = fwidth( len2 );

      if ( abs( vUv.y ) > 1.0 ) 
      {
        alpha = 1.0 - smoothstep( 1.0 - dlen, 1.0 + dlen, len2 );
      }
    #else
      if( abs( vUv.y ) > 1.0 )
      {
        float a = vUv.x;
        float b = ( vUv.y > 0.0 ) ? vUv.y - 1.0 : vUv.y + 1.0;
        float len2 = a * a + b * b;

        if ( len2 > 1.0 ) discard;
      }
    #endif
  #endif

  #ifdef USE_VERTEX_COLORS
    col = vColor;
  #endif

  col = vec3( 0.0 );

  frag_color = vec4( col, alpha );
}
