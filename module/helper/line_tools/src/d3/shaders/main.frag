#version 300 es
precision highp float;

// #include <defines>

uniform vec3 u_color;
uniform float u_width;

uniform float u_dash_offset;
uniform float u_dash_size;
uniform float u_dash_gap;

in vec2 vUv;
in vec3 vViewPos;
in vec3 vViewA;
in vec3 vViewB;

#ifdef USE_VERTEX_COLORS
  in vec3 vColor;
#endif

#ifdef USE_DASHES
  in float vLineDistance;
  in float vLineDistanceA;
  in float vLineDistanceB;
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

void main()
{
  float alpha = 1.0;
  vec3 col = u_color;

  // #ifdef USE_DASHES
  //   //if( vUv.y < -1.0 || vUv.y > 1.0 ) { discard; }
  //   float dashCoverage = mod( vLineDistance + u_dash_offset, u_dash_gap + u_dash_size );
  //   float distanceA = max( vLineDistanceA, vLineDistance - dashCoverage );
  //   float distanceB = min( vLineDistanceB, vLineDistance - ( dashCoverage - u_dash_size ) );

  //   if( dashCoverage > u_dash_size )
  //   {
  //     distanceA += u_dash_gap + u_dash_size;
  //     distanceB += u_dash_gap + u_dash_size;
  //   }

  //   vec3 lineStart = mix( vViewA, vViewB, ( distanceA - vLineDistanceA ) / ( vLineDistanceB - vLineDistanceA ) );
  //   vec3 lineEnd =  mix( vViewA, vViewB, ( distanceB - vLineDistanceA ) / ( vLineDistanceB - vLineDistanceA ) );

  //   //if( mod( vLineDistance + dashOffset, dashGap + dashSize ) > dashSize ) { discard; }
  // #else
    vec3 lineStart = vViewA;
    vec3 lineEnd = vViewB;
  //#endif

  #ifdef USE_WORLD_UNITS

    #ifdef USE_DASHES
      float viewDirection = sign( dot( vViewPos, vViewB - vViewA ) );
      //if( viewDirection == 0.0 ) 
      { viewDirection = 1.0; }
      float dashCoverage = mod( vLineDistance + u_dash_offset, u_dash_gap + u_dash_size );

      float distanceA1 = clamp( vLineDistance - dashCoverage, vLineDistanceA, vLineDistanceB );
      float distanceB1 = clamp( vLineDistance - ( dashCoverage - u_dash_size ), vLineDistanceA, vLineDistanceB );
      vec3 lineStart1 = mix( vViewA, vViewB, ( distanceA1 - vLineDistanceA ) / ( vLineDistanceB - vLineDistanceA ) );
      vec3 lineEnd1 =  mix( vViewA, vViewB, ( distanceB1 - vLineDistanceA ) / ( vLineDistanceB - vLineDistanceA ) );

      float distanceA2 = clamp( distanceA1 + viewDirection * ( u_dash_size + u_dash_gap ), vLineDistanceA, vLineDistanceB );
      float distanceB2 = clamp(  distanceB1 + viewDirection * ( u_dash_size + u_dash_gap ), vLineDistanceA, vLineDistanceB );
      vec3 lineStart2 = mix( vViewA, vViewB, ( distanceA2 - vLineDistanceA ) / ( vLineDistanceB - vLineDistanceA ) );
      vec3 lineEnd2 =  mix( vViewA, vViewB, ( distanceB2 - vLineDistanceA ) / ( vLineDistanceB - vLineDistanceA ) );

      vec3 rayEnd = normalize( vViewPos ) * 1e5;
      vec3 lineDir1 = lineEnd1 - lineStart1;
      vec3 lineDir2 = lineEnd2 - lineStart2;

      vec2 params1 = closestLineToLine( lineStart1, lineEnd1, vec3( 0.0, 0.0, 0.0 ), rayEnd );
      vec2 params2 = closestLineToLine( lineStart2, lineEnd2, vec3( 0.0, 0.0, 0.0 ), rayEnd );

      vec3 p11 = lineStart1 + lineDir1 * params1.x;
      vec3 p12 = rayEnd * params1.y;
      vec3 delta1 = p11 - p12;
      float len1 = length( delta1 );
      float norm1 = len1 / u_width;

      vec3 p21 = lineStart2 + lineDir2 * params2.x;
      vec3 p22 = rayEnd * params2.y;
      vec3 delta2 = p21 - p22;
      float len2 = length( delta2 );
      float norm2 = len2 / u_width;

      float norm = min( norm1, norm2 );


      #ifdef USE_ALPHA_TO_COVERAGE
        float dnorm = fwidth( norm );
        alpha = 1.0 - smoothstep( 0.5 - dnorm, 0.5 + dnorm, norm );
      #else
        if ( norm > 0.5 ) { discard; }
      #endif
    #else
      vec3 rayEnd = normalize( vViewPos ) * 1e5;
      vec3 lineDir = lineEnd - lineStart;

      vec2 params = closestLineToLine( lineStart, lineEnd, vec3( 0.0, 0.0, 0.0 ), rayEnd );

      vec3 p1 = lineStart + lineDir * params.x;
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
