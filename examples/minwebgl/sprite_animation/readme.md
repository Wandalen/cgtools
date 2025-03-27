# Sprite animation
A sprite animation of rock example, which loads a sprite sheet (an image with many frames of animation), splits it into individual sprites, and plays the animation by switching between them.

## Showcase
![](./showcase.gif)

## How is it useful
- Fast sprite loading one by one (texture array).
- Get element of sprite by it's depth index.

## How to run
To run the example you need to go to the directory of this example:
```bash
cd examples/minwebgl/sprite_animation/
```

Next step its just serve it:
```bash
trunk serve --release
```

If you need change on your own sprite sheet:
1. Put your sprite sheet in assets folder of the example.
2. Change variables in main.rs:
```rust
let path = "static/your_sprite_sheet.png";
let sprties_in_row = 8; // amount of elements in each row.
let sprite_width = 128; // width of element of sprite sheet.
let sprite_height = 128; // height of element of sprite sheet.
let amount = 64; // count of all elements in sprite sheet.
let frame_rate = 24.0; // on your opinion.
```
3. Run it :)
