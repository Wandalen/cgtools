## Mapgen tile rendering

This example shows how create own tile renderer using `WebGl2`. Goal of this example is render tilemap on quad.

![showcase](./resources/showcase.png)

### How it is useful

The example shows:
- how use textures with unsigned integer formats, textures without mipmap filtering;
- how store images as DOM elements and get access to them by their id;
- how create textures from raw data in array;
- how use more than one texture.

### How it works

Example include such noteworthy steps:
 - load tileset image;
 - prepare vertex positions, uvs with VAO;
 - prepare tileset texture array;
 - prepare tilemap texture with raw data;
 - draw.

Important part of resource preparation is texture binding. Remember use this code for binding texture to slot:

``` rust
  let texture = gl.create_texture();
  gl.active_texture(texture_id);
  gl.bind_texture( GL::TEXTURE_2D, texture.as_ref() ); 
```

And this code for binding uniform texture location to certain slot:

``` rust
  let texture_location = gl.get_uniform_location( &program, "texture" );
  gl.uniform1i(texture_location.as_ref(), 0);
```

Fragment shader do all magic for placing each tile on its place: 
 1. for each fragment we get translate quad uv to tile uv by get fract part of current uv divided on tile size ((1 / tilemap_width); (1 / tilemap_height));
 2. get tile id for current fragment from tilemap;
 3. get texel from tileset texture by tile coords from 1. step and tile id from 2. step. 

### Running

Make sure you have installed all the necessary [dependencies](../../../module/min/minwebgl/readme.md)
In order to run the example navigate to example's directory and run next command:
``` bash
trunk serve
```
If you want to load own tile set image, upload it into `resources` folder as `tileset.png` and set tile count in const LAYERS. 

``` rust
const LAYERS: i32 = 6;
```

Tileset image must store tile textures from up to down order, all textures must have equal size without align.

If you want make own tile maps, change `DATA` const. Tile map must be square (with equal height and width). Tile map consist of tile ids. Tile id is texture position in tile set from up to down.

``` rust
const DATA: [u8; 16] = [
  0,0,1,2,
  0,1,2,3,
  1,2,3,4,
  2,3,4,5
];
```