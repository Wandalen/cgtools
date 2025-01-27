#version 300 es
precision mediump float;
precision mediump sampler2DArray;
precision mediump usampler2D;

in vec2 v_uv;
out vec4 color;

uniform usampler2D map_sampler;
uniform sampler2DArray tiles_sampler;
uniform vec2 texel_size;

// Quad contain grid of tiles. That defined as part of quad texture.
void main(){
    // Fract part of uv for current fragment is local coords for certain tile
    vec2 tile_coords = fract(v_uv / texel_size);
    // map_sampler gets tile type from discrete texture. For indexing quad uv are used. 
    // Every texel of map texture can contain value from 0 to 255, that stored in r channel.
    uint tile_id = texture( map_sampler, v_uv ).r;
    // With tile_coord and tile_id we can choose tile texture that stored in 2D texture array.
    color = texture(tiles_sampler, vec3(tile_coords, tile_id));
}
