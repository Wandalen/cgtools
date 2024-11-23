#version 300 es
#pragma vscode_glsllint_stage : vert

layout( location = 0 ) in vec4 offset;
layout( location = 1 ) in vec4 uv_info;
layout( location = 2 ) in vec2 size;

uniform mat4x4 viewMatrix;
uniform mat4x4 projectionMatrix;
uniform vec2 texSize;
uniform vec4 boundingBox;

const vec2 points[ 4 ] = vec2[]
(
  vec2( 0.0, -1.0 ),   
  vec2( 1.0, -1.0 ),    
  vec2( 0.0, 0.0 ),   
  vec2( 1.0, 0.0 )   
);

const vec2 uvs[ 4 ] = vec2[]
(
  vec2( 0.0, 0.0 ),  
  vec2( 1.0, 0.0 ),  
  vec2( 0.0, 1.0 ),  
  vec2( 1.0, 1.0 ) 
);

out vec2 vUv;
out vec3 vPos; 

void main()
{
  vec2 letter_offset = offset.xy;
  vec2 string_offset = offset.zw;
  vec2 uv = uv_info.xy;
  vec2 uv_extent = uv_info.zw;

  vec2 scale = 20.0 / texSize;
  vec2 centerOffset = -boundingBox.xy - ( boundingBox.zw - boundingBox.xy ) / 2.0;

  vec2 w_pos = points[ gl_VertexID ] * size;
  w_pos += centerOffset;
  w_pos += letter_offset;
  w_pos += string_offset;
  w_pos *= scale;

  vUv = uv + vec2( gl_VertexID % 2, 1 - gl_VertexID / 2 ) * uv_extent;
  vPos = vec3( w_pos, 0.0 );
  gl_Position = projectionMatrix * viewMatrix * vec4( w_pos, 0.0, 1.0 );
}
