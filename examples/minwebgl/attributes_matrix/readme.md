# Matrix Attributes

**Keywords:** Matrices, Transformations, WebGL2, Attributes

This demo demonstrates passing matrix data as vertex attributes in WebGL2. Since matrices are larger than typical attributes, this example shows how to split them across multiple attribute slots and reconstruct them in shaders.

This technique is particularly useful for skinned animation or instanced rendering where each instance needs its own transformation matrix. It enables efficient batch rendering of objects with unique transforms.

![image](./showcase.gif)

**[How to run](../how_to_run.md)**

**References:**

* [WebGL 2: Matrix Attributes]

[WebGL 2: Matrix Attributes]: https://www.youtube.com/watch?v=8XOctnNrJn4
