#version 300 es
precision mediump float;
precision mediump sampler2DArray;
precision mediump usampler2D;

in vec2 v_uv;
out vec4 color;

uniform usampler2D map_sampler;
uniform sampler2DArray tiles_sampler;
uniform vec2 texel_size;

void main(){
  vec2 tile_coords = fract(v_uv / texel_size);
  uint tile_id = texture( map_sampler, v_uv ).r;
  color = texture(tiles_sampler, vec3(tile_coords, tile_id));
}