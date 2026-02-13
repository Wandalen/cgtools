#version 300 es
// Renders 3d line, supporting both screen space and world space units.
// Allows for anti-aliasing with alpha-to-coverage enabled.
// Has an optional color attribute for the points of the line.
precision highp float;

// #include <defines>

layout( location = 0 ) in vec2 position;
layout( location = 1 ) in vec2 uv;
layout( location = 2 ) in vec3 inPointA;
layout( location = 3 ) in vec3 inPointB;

#ifdef USE_VERTEX_COLORS
  layout( location = 4 ) in vec3 colorA;
  layout( location = 5 ) in vec3 colorB;
#endif

#ifdef USE_DASH
  layout( location = 6 ) in float distanceA;
  layout( location = 7 ) in float distanceB;
#endif

uniform mat4 u_world_matrix;
uniform mat4 u_view_matrix;
uniform mat4 u_projection_matrix;

uniform vec2 u_resolution;
uniform float u_width;

out vec2 vUv;
out vec3 vViewPos;
out vec3 vViewA;
out vec3 vViewB;

#ifdef USE_VERTEX_COLORS
  out vec3 vColor;
#endif

#ifdef USE_DASH
  out float vLineDistance;
  flat out float vLineDistanceA;
  flat out float vLineDistanceB;
#endif

void trimSegment( const in vec3 start, inout vec3 end )
{

  // trim end segment so it terminates between the camera plane and the near plane

  // conservative estimate of the near plane
  float a = u_projection_matrix[ 2 ][ 2 ]; // 3nd entry in 3th column
  float b = u_projection_matrix[ 3 ][ 2 ]; // 3nd entry in 4th column
  float nearEstimate = - 0.5 * b / a;

  float alpha = ( nearEstimate - start.z ) / ( end.z - start.z );

  end = mix( start, end, alpha );

}

void main() 
{
  vec3 viewA = ( u_view_matrix * u_world_matrix * vec4( inPointA, 1.0 ) ).xyz;
  vec3 viewB = ( u_view_matrix * u_world_matrix * vec4( inPointB, 1.0 ) ).xyz;

  bool perspective = ( u_projection_matrix[ 2 ][ 3 ] == - 1.0 ); // 4th entry in the 3rd column

  if ( perspective ) 
  {
    if ( viewA.z < 0.0 && viewB.z >= 0.0 ) 
    {
      trimSegment( viewA, viewB );
    } 
    else if ( viewB.z < 0.0 && viewA.z >= 0.0 ) 
    {
      trimSegment( viewB, viewA );
    }
  }

  float aspect = u_resolution.x / u_resolution.y;

  vec4 clipA = u_projection_matrix * vec4( viewA, 1.0 );
  vec4 clipB = u_projection_matrix * vec4( viewB, 1.0 );

  vec4 clip = vec4( 0.0 );

  #ifdef USE_WORLD_UNITS
    vec3 viewPos = position.y < 0.5 ? viewA : viewB;
    vec3 viewAB = normalize( viewB - viewA );
    vec3 midForward = normalize( mix( viewA, viewB, 0.5 ) );
    vec3 up = normalize( cross( viewAB, midForward ) );
    vec3 right = normalize( cross( viewAB, up ) );

    float halfWith = 0.5 * u_width;

    // Protrude vertices to create an illusion of 3d shape in view space
    viewPos += position.x < 0.0 ? up * halfWith : -up * halfWith;

    //#ifndef USE_DASH
      viewPos += position.y < 0.5 ? -halfWith * viewAB : halfWith * viewAB;
      viewPos += right * halfWith;
      if( position.y < 0.0 || position.y > 1.0 )
      {
        viewPos += 2.0 * -right * halfWith;
      }
    //#endif
    
    clip = u_projection_matrix * vec4( viewPos, 1.0 );
    vec3 ndcShift = position.y < 0.5 ? clipA.xyz / clipA.w : clipB.xyz / clipB.w;
    clip.z = ndcShift.z * clip.w;

    vViewPos = viewPos;
  #else // Screen space units
    vec2 screenA = u_resolution * ( 0.5 * clipA.xy / clipA.w + 0.5 );
    vec2 screenB = u_resolution * ( 0.5 * clipB.xy / clipB.w + 0.5 );

    vec2 xBasis = normalize( screenB - screenA );
    vec2 yBasis = vec2( -xBasis.y, xBasis.x );
    yBasis *= position.x;

    vec2 basis = yBasis;
    if ( position.y < 0.0 ) 
    {
      basis -= xBasis;
    } 
    else if( position.y > 1.0 )
    {
      basis += xBasis;
    }

    vec2 screenP = ( position.y < 0.5 ) ? screenA : screenB;
    vec2 p = screenP + u_width * basis;


    clip = ( position.y < 0.5 ) ? clipA : clipB;
    
    clip.xy = clip.w * ( 2.0 * p / u_resolution - 1.0 );

  #endif

  #ifdef USE_DASH
    vLineDistance = position.y < 0.5 ? distanceA : distanceB;
    vLineDistanceA = distanceA;
    vLineDistanceB = distanceB;
  #endif

  vUv =  uv;
  vViewA = viewA;
  vViewB = viewB;

  #ifdef USE_VERTEX_COLORS
    vColor = mix( colorA, colorB, step( 0.5, position.y ) );
  #endif  

  gl_Position = clip;
}