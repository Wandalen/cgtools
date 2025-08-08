- [OpenFOAM](#openfoam)  


## [OpenFOAM](https://www.openfoam.com/)

OpenFOAM( Field Operation and Manipulation ) - is a Computational Fluid Dynamics software, that provides with solvers for physical simulations. It does not provide the visualization. For that you need to use another software like Paraview.

- [Documentation](https://www.openfoam.com/documentation/overview)
- [User Guide](https://dl.openfoam.com/source/latest/UserGuide.pdf)
- [Tutorial](https://dl.openfoam.com/source/latest/TutorialGuide.pdf)
- [Repository](https://develop.openfoam.com/Development/openfoam)

Before working with the openFOAM, you need to set it up by running the following command
```sh
source /installation/path/OpenFOAM-v2506/etc/bashrc
```

For each simulations( case ) a seperate directory needs to be created, where you specify the simulation properties in files, that contain key value pair in plane text format

-   `constant` directory - for geometric and physical properties
-   `system` directory - settings of the simulation
-   `0`(zero) directory - for field and boundary conditions at timestemp zero( For each property in a seperate file )

OpenFOAM has helper programs to make it easier to create your scene. For example the `blockMeshDict` file explains to the `blockMesh` utility how subdivide the space into cells and boundary from which the mesh will be created.

After you are done with specifying conditions for the scene, you run the solver for your specific task, which write to file the state of the field at different timesteps.  

This result can then be viewed and further exported using paraview. To view the result you just need to create the `case.foam` placeholder file and load it in the paraview. The builtin openFOAM reader will the handle loading the rest of the data.



[VTK]: https://docs.vtk.org/en/latest/getting_started/index.html