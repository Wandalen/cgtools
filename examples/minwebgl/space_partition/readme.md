# Space partitioning

**Keywords:** KdTree, KNN, 3D Models, WebGL2

**Technologies:** WebGL2, CGTools, KdTree, KNN

This demo uses `spart` crate to find points closest to the mouse cursor. The neighbouring points are coloured green. The neighbour search is displayed using red lines/circle

## Controls

- **Search type**: Switch between KNN (K-nearest neighbors) and Range (radius-based) search
- **K Neighbours**: Adjust number of closest points to find (KNN mode only)
- **Range radius**: Adjust search radius for finding nearby points (Range mode only)

Move your mouse over the canvas to see the spatial search in action.

![image](./showcase.jpg)

**[How to run](../how_to_run.md)**

**References:**

* [KdTree]


[KdTree]: https://en.wikipedia.org/wiki/K-d_tree

