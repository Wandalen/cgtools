- [VTK](#vtk)

## [VTK]
Visualization toolkit - is an open source a C++ library for data visualization. It is also available for use with Python, Java, C#, Javascript and WebAssembly.   
It contains a large collection of algorithm and filter for processsing data.

`Process` - transforms data in some way
- **Source** - 0 inputs, >=1 outputs
- **Filter** - >=1 inputs, >=1 outputs
- **Mapper** - >=1 inputs, 0 outputs

To process a large dataset, the dataset is required to be
- **Separable** - The data can be broken into pieces
- **Mappable** - An ability to determine what portion of the input data is required to generate a given portion of the output
- **Result invariant** - The results should be independent of the number of pieces and independent of the execution model( single- or multi-threaded )

`Readers/Writes`
- **Reader** - ingest data from a file, create a data object and the pass the object down the pipeline for processing
- **Writer** - ingest data object and then write the data object to a file.

**Examples**: vtkSTLReader, vtkBYUWriter,

`Importers/Exporters`
- **Importer** - restores an entire scene
- **Exporter** - saves an entire scene

**Examples**: vtk3DSImporter, vtkVRMLExporter,

To satisfy the third requirement, an ability to generate boundary data, or *ghost cells*, is required
Links:
- [VTK]
- [User guide]
- [Book]
- [Documentation]
- [Examples]
- [VTK with EGUI demo]

[VTK]: https://docs.vtk.org/en/latest/getting_started/index.html
[User guide]: https://vtk.org/wp-content/uploads/2021/08/VTKUsersGuide.pdf
[Book]: https://book.vtk.org/en/latest/
[Documentation]: https://docs.vtk.org/en/latest/about.html
[Examples]: https://examples.vtk.org/site/
[VTK with EGUI demo]: https://github.com/Gerharddc/vtk-egui-demo/tree/main