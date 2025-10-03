#version 300 es
precision highp float;

// #include <defines>

const float MAX_FLOAT = 3.402823466e+38;

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

 #ifdef USE_DASHES
  float getDistanceToDash( vec3 rayEnd, float i )
  {
    float dashCoverage = mod( vLineDistance + u_dash_offset, u_dash_gap + u_dash_size );

    float distanceA = vLineDistance - dashCoverage;
    float distanceB = distanceA + u_dash_size;

    distanceA += i * ( u_dash_size + u_dash_gap );
    distanceB += i * ( u_dash_size + u_dash_gap );

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
    #ifdef USE_DASHES
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
