## Interactive filters

This example demonstrates work of different image filters implemented in WebGl2.

![](showcase.gif)

### How it is useful

This demo shows how to implement different filters alongside different techniques such as
kernel convolutoin and two-pass rendering in WebGl2.

### How to run

Make sure you have installed all the necessary [dependencies](../../../module/min/minwebgl/readme.md)
In order to run the example navigate to example's directory and run next command:

``` bash
trunk serve
```

If you want to load your own image, upload it into `resources` folder
and then provide its path into `image_path` variable.

``` rust
let image_path = "static/your_image.format";
```

You can select filter in the list on the left side and adjust its parameters in the small
window in the top right corner if they are present.
