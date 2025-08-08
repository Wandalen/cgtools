## Wave function collapse

This example shows how create map with wave function collapse algorithm and display it with tile renderer using `WebGl2`. Goal of this example is generate tile map using wave function collapse algorithm.

<p align="center">
  <img src="./showcase.png" width="600px">
</p>

### How it is useful

The example shows:
- how wave function collapse algorithm works.
- how use implemented algorithm for generating tile map.
- how convert raw tile map data to texture. 

Rendering details description can be found in [ mapgen_tiles_rendering ]( ../mapgen_tiles_rendering/readme.md ) example

### How it works

Example include such noteworthy steps:
 - set size, front, relations fields with related methods of Wfc.
 - call calculate method.
 - calculate method do repeatedly cycle collapse-propagate while front isn't empty.
    - collapse - for each tile coordinates with minimal entropy in front choose any posible variant and write result back to map.
    - propagate - set new front and update front tiles variants.
 - when the cycle ended check and handle errors for each tile and then returns tile map.

### Running

Make sure you have installed all the necessary [ dependencies ]( ../../../module/min/minwebgl/readme.md )
In order to run the example navigate to example's directory and run next command:
``` bash
trunk serve
```
If you want to load own tile set image, upload it into `resources` folder as `tileset.png` and set tile count in const LAYERS. 

``` rust
const LAYERS : i32 = 6;
```

Tileset image must store tile textures from up to down order, all textures must have equal size without align.

You can change tile map size with SIZE const. If width don't be equal height than tiles will be rectangles not squares stretched along quad. 

``` rust
const SIZE : ( usize, usize ) = ( 32, 32 );
```

If you want set own adjacent relations between tiles, you can change RELATIONS const. List depends from tile variants count. Each items id in RELATIONS equals to tile id. Each item contains list of possible adjacent tiles for current tile id. Changing RELATIONS, LAYERS, SIZE, `tileset.png` texture may create new patterns. 

``` rust
const RELATIONS : &str = "
  [
    [ 0, 1 ],
    [ 1, 2 ],
    [ 2, 3 ],
    [ 3, 4 ],
    [ 4, 5 ],
    [ 5 ]
  ]
";
```