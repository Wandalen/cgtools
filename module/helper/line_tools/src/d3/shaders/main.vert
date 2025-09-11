#version 300 es
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

void main() 
{
  vec3 viewA = ( u_view_matrix * u_world_matrix * vec4( inPointA, 1.0 ) ).xyz;
  vec3 viewB = ( u_view_matrix * u_world_matrix * vec4( inPointB, 1.0 ) ).xyz;

  float aspect = u_resolution.x / u_resolution.y;

  vec4 clipA = u_projection_matrix * vec4( viewA, 1.0 );
  vec4 clipB = u_projection_matrix * vec4( viewB, 1.0 );

  vec3 ndcA = clipA.xyz / clipA.w;
  vec3 ndcB = clipB.xyz / clipB.w;

  vec2 ndcDir = normalize( ndcB.xy - ndcA.xy );
  // Adjust for aspect ratio
  ndcDir.x *= aspect;
  ndcDir = normalize( ndcDir );

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
    viewPos += position.y < 0.5 ? -halfWith * viewAB : halfWith * viewAB;
    viewPos += right * halfWith;
    if( position.y < 0.0 || position.y > 1.0 )
    {
      viewPos += 2.0 * -right * halfWith;
    }
    
    clip = u_projection_matrix * vec4( viewPos, 1.0 );
    vec3 ndcShift = position.y < 0.5 ? ndcA : ndcB;
    clip.z = ndcShift.z * clip.w;

    vViewPos = viewPos;
  #else
    vec2 ndcOffset = vec2( ndcDir.y, -ndcDir.x );
    ndcDir.x /= aspect;
    ndcOffset.x /= aspect;

    if ( position.x < 0.0 ) ndcOffset *= - 1.0;

    if ( position.y < 0.0 ) 
    {
      ndcOffset += -ndcDir;
    } 
    else if ( position.y > 1.0 ) 
    {
      ndcOffset += ndcDir;
    }

    ndcOffset *= u_width;
    clip = ( position.y < 0.5 ) ? clipA : clipB;
    
    vec2 clipOffset = ndcOffset * clip.w;
    clip.xy += clipOffset;

  #endif

  vUv = uv;
  vViewA = viewA;
  vViewB = viewB;

  #ifdef USE_VERTEX_COLORS
    vColor = mix( colorA, colorB, step( 0.5, position.y ) );
  #endif  

  gl_Position = clip;
}