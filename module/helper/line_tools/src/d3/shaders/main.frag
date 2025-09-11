#version 300 es
precision highp float;

// #include <defines>

uniform vec3 u_color;
uniform float u_width;

in vec2 vUv;
in vec3 vViewPos;
in vec3 vViewA;
in vec3 vViewB;

#ifdef USE_VERTEX_COLORS
  in vec3 vColor;
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

  vec3 rayEnd = normalize( vViewPos.xyz ) * 1e5;
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

  #ifdef USE_VERTEX_COLORS
    col = vColor;
  #endif

  col = vec3( 0.0 );
  col = vec3( vUv, 0.0 );

  frag_color = vec4( col, alpha );
}
