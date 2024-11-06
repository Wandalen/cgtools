## Raycasting

This example demonstrates basics of raycast-driven rendering algorithm used in Wolfenstein 3D.

### How it is useful

This demo implements DDA raycasting [algorithm](https://lodev.org/cgtutor/raycasting.html) and
shows how to use it for drawing a scene.

### How to run

Make sure you have installed all the necessary [dependencies](../../../module/min/minwebgl/readme.md)
In order to run the example navigate to example's directory and run next command:
``` bash
trunk serve
```

To navigate use WASD keys as W and S for moving forward and backward, A and D to turn left and right.
You can adjust amount of rays to see how it affects output image.
Find the next line in code and change value as you desire:
``` rust
let ray_count = 120;
```

![](showcase.png)
