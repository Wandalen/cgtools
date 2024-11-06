# Video as texture
This example shows how to make a texture as a video.

## Showcase
![](./showcase.gif)

## How is it useful
- Upload texture as a video with any resolution.
- Texture playing automaticaly.

## How to run
To run the example you need to go to the directory of this example:
```bash
cd examples/minwebgl/video_as_texture/
```

Next step its just serve it:
```bash
trunk serve --release
```

## How to change on your own video
1. Put your video in assets folder of the example.
2. Change variables in main.rs:
```rust
let path = "static/your_video.mp4";
let video_width = 640;  // video width parameter
let video_height = 480; // video height parameter
//...
let data : [ f32; 16 ] = // these vertices coords for rectange on the middle of canvas
[//x     y     t_x   t_y
  -0.3,  0.5,  0.0,  0.0,
   0.3,  0.5,  1.0,  0.0,
  -0.3, -0.5,  0.0,  1.0,
   0.3, -0.5,  1.0,  1.0,
];
```
4. Run it:
```bash
trunk serve --release
```
