#version 300 es
precision mediump float;

uniform sampler2D equi_texture;

out vec4 frag_color;
in vec3 localPos;

const vec2 normalizer = vec2( 0.15915, 0.3183 );
void main()
{
  vec3 dir = normalize( localPos );

  float longitude = asin( dir.y );
  float latitude = atan( dir.z, dir.x );

  vec2 uv = vec2( latitude, longitude ) * normalizer;
  vec4 color = texture( equi_texture, uv );
  frag_color =  vec4( color.rgb, 1.0 );
}