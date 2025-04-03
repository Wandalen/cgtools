#![allow(clippy::collapsible_else_if)] // Allows for cleaner structure in some cases

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    HtmlCanvasElement, WebGl2RenderingContext as GL, WebGlUniformLocation,
};
use glow::{
    Context as GlowContext, HasContext, Buffer, Program, Texture, VertexArray, Framebuffer
};
use std::rc::Rc;
use nalgebra_glm as glm; // Using nalgebra-glm for math

// --- Constants ---
const MODEL_BYTES: &[u8] = include_bytes!("./assets/model.glb"); // Adjust path if needed

// --- Logging ---
fn log(s: &str) {
    web_sys::console::log_1(&s.into());
}

// --- Error Handling ---
#[derive(Debug)]
enum AppError {
    WebGlContext,
    ShaderCompile(String),
    ProgramLink(String),
    ResourceCreation(String),
    FramebufferIncomplete(String),
    MissingExtension(String),
    GltfParse(String),
    GltfLogic(String),
    BufferCreationFailed,
    VaoCreationFailed,
    TextureCreationFailed,
    FramebufferCreationFailed,
    ProgramCreationFailed,
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for AppError {}

// --- Shaders ---

// Pass 1: 3D Object Rendering
const OBJECT_VS: &str = include_str!("./shaders/object.vert");

const OBJECT_FS: &str = include_str!("./shaders/object.frag");

// Pass 2 & 3 & 4: Fullscreen Quad Vertex Shader
const FULLSCREEN_VS: &str = include_str!("./shaders/fullscreen.vert");

// Pass 2: JFA Initialization Fragment Shader
const JFA_INIT_FS: &str = include_str!("./shaders/jfa_init.frag");

// Pass 3: JFA Step Fragment Shader
const JFA_STEP_FS: &str = include_str!("./shaders/jfa_step.frag");

// Pass 4: Final Outline Composite Fragment Shader
const OUTLINE_FS: &str = include_str!("./shaders/outline.frag");

// --- WebGL Helper Functions ---

fn create_shader(gl: &GlowContext, shader_type: u32, source: &str) -> Result<glow::Shader, AppError> {
    unsafe {
        let shader = gl.create_shader(shader_type).map_err(|e| AppError::ResourceCreation(format!("Shader: {}", e)))?;
        gl.shader_source(shader, source);
        gl.compile_shader(shader);
        if !gl.get_shader_compile_status(shader) {
            let log = gl.get_shader_info_log(shader);
            gl.delete_shader(shader); // Clean up on failure
            Err(AppError::ShaderCompile(log))
        } else {
            Ok(shader)
        }
    }
}

fn create_program(gl: &GlowContext, vs_source: &str, fs_source: &str, attrib_locations: Option<&[(u32, &str)]>) -> Result<Program, AppError> {
    unsafe {
        let program = gl.create_program().map_err(|_| AppError::ProgramCreationFailed)?;
        let vs = create_shader(gl, GL::VERTEX_SHADER, vs_source)?;
        let fs = create_shader(gl, GL::FRAGMENT_SHADER, fs_source)?;

        gl.attach_shader(program, vs);
        gl.attach_shader(program, fs);

        // Bind attribute locations BEFORE linking (important!)
        if let Some(attribs) = attrib_locations {
            for (loc, name) in attribs {
                gl.bind_attrib_location(program, *loc, name);
                 log(&format!("Binding attrib location {} to '{}'", loc, name));
            }
        }

        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            let log = gl.get_program_info_log(program);
            // Detach and delete shaders and program on failure
            gl.detach_shader(program, vs);
            gl.detach_shader(program, fs);
            gl.delete_shader(vs);
            gl.delete_shader(fs);
            gl.delete_program(program);
            Err(AppError::ProgramLink(log))
        } else {
            // Detach and delete shaders after successful link
            gl.detach_shader(program, vs);
            gl.detach_shader(program, fs);
            gl.delete_shader(vs);
            gl.delete_shader(fs);
            Ok(program)
        }
    }
}

// Creates a texture suitable for rendering to (color or depth)
fn create_render_target_texture(
    gl: &GlowContext,
    width: i32,
    height: i32,
    internal_format: u32, // e.g., GL::RGBA8, GL::RGBA32F, GL::DEPTH_COMPONENT24
    format: u32,          // e.g., GL::RGBA, GL::DEPTH_COMPONENT
    pixel_type: u32,      // e.g., GL::UNSIGNED_BYTE, GL::FLOAT, GL::UNSIGNED_INT
) -> Result<Texture, AppError> {
    unsafe {
        let texture = gl.create_texture().map_err(|_| AppError::TextureCreationFailed)?;
        gl.bind_texture(GL::TEXTURE_2D, Some(texture));
        gl.tex_image_2d(
            GL::TEXTURE_2D,
            0,
            internal_format as i32, // glow's API expects i32 here
            width,
            height,
            0,
            format,
            pixel_type,
            None, // No initial data needed for render target
        );

        // Use NEAREST filtering for JFA/depth, LINEAR might be ok for final color display
        let filter = if format == GL::DEPTH_COMPONENT { GL::NEAREST } else { GL::NEAREST }; // Or GL::LINEAR for color?
        gl.tex_parameter_i32(GL::TEXTURE_2D, GL::TEXTURE_MIN_FILTER, filter as i32);
        gl.tex_parameter_i32(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, filter as i32);
        gl.tex_parameter_i32(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::CLAMP_TO_EDGE as i32);
        gl.tex_parameter_i32(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);

        gl.bind_texture(GL::TEXTURE_2D, None); // Unbind
        Ok(texture)
    }
}

// Creates a Framebuffer Object (FBO) linking color and optionally depth textures
fn create_framebuffer(gl: &GlowContext, color_texture: Texture, depth_texture: Option<Texture>) -> Result<Framebuffer, AppError> {
    unsafe {
        let fb = gl.create_framebuffer().map_err(|_| AppError::FramebufferCreationFailed)?;
        gl.bind_framebuffer(GL::FRAMEBUFFER, Some(fb));

        // Attach color texture
        gl.framebuffer_texture_2d(
            GL::FRAMEBUFFER,
            GL::COLOR_ATTACHMENT0,
            GL::TEXTURE_2D,
            Some(color_texture),
            0,
        );

        // Attach depth texture if provided
        if let Some(depth_tex) = depth_texture {
            gl.framebuffer_texture_2d(
                GL::FRAMEBUFFER,
                GL::DEPTH_ATTACHMENT,
                GL::TEXTURE_2D,
                Some(depth_tex),
                0,
            );
        }

        // Check if framebuffer is complete
        let status = gl.check_framebuffer_status(GL::FRAMEBUFFER);
        if status != GL::FRAMEBUFFER_COMPLETE {
            let status_str = match status {
                 GL::FRAMEBUFFER_UNSUPPORTED => "UNSUPPORTED",
                 GL::FRAMEBUFFER_INCOMPLETE_ATTACHMENT => "INCOMPLETE_ATTACHMENT",
                 GL::FRAMEBUFFER_INCOMPLETE_MISSING_ATTACHMENT => "INCOMPLETE_MISSING_ATTACHMENT",
                 GL::FRAMEBUFFER_INCOMPLETE_MULTISAMPLE => "INCOMPLETE_MULTISAMPLE",
                 GL::FRAMEBUFFER_INCOMPLETE_DIMENSIONS => "INCOMPLETE_DIMENSIONS",
                 _ => "Unknown",
             };
             gl.bind_framebuffer(GL::FRAMEBUFFER, None); // Unbind before deleting
             gl.delete_framebuffer(fb);
             Err(AppError::FramebufferIncomplete(format!("Status: {} ({})", status_str, status)))
        } else {
            gl.bind_framebuffer(GL::FRAMEBUFFER, None); // Unbind on success
            Ok(fb)
        }
    }
}

// --- GLTF Data Structures ---

#[derive(Debug)] // Add Debug trait for logging/errors
struct Primitive {
    vao: VertexArray,
    // index_buffer is implicitly bound to VAO, no need to store separately unless for cleanup verification
    index_count: i32,
    index_type: u32, // e.g., GL::UNSIGNED_SHORT, GL::UNSIGNED_INT
    mode: u32,       // e.g., GL::TRIANGLES
    has_indices: bool,
}

#[derive(Debug)]
struct Mesh {
    primitives: Vec<Primitive>,
    name: Option<String>,
}

#[derive(Debug)]
struct Node {
    mesh_index: Option<usize>,
    transform: glm::Mat4,
    children: Vec<usize>,
    name: Option<String>,
}

#[derive(Debug)]
struct GltfModel {
    meshes: Vec<Mesh>,
    nodes: Vec<Node>,
    root_nodes: Vec<usize>,
    // Store all GL buffers (VBOs/EBOs) flatly for easier cleanup
    gl_buffers: Vec<Buffer>,
    // Store VAOs flatly for cleanup? No, VAOs are in Primitives.
}


// --- App State Structure ---

struct GlResources {
    // Programs
    object_program_3d: Program,
    jfa_init_program: Program,
    jfa_step_program: Program,
    outline_program: Program,

    // Uniform locations (using Option<...> is good practice)
    object_u_projection: Option<WebGlUniformLocation>,
    object_u_view: Option<WebGlUniformLocation>,
    object_u_model: Option<WebGlUniformLocation>,
    jfa_init_u_object_tex: Option<WebGlUniformLocation>,
    jfa_init_u_resolution: Option<WebGlUniformLocation>,
    jfa_step_u_jfa_tex: Option<WebGlUniformLocation>,
    jfa_step_u_resolution: Option<WebGlUniformLocation>,
    jfa_step_u_step_size: Option<WebGlUniformLocation>,
    outline_u_object_tex: Option<WebGlUniformLocation>,
    outline_u_jfa_tex: Option<WebGlUniformLocation>,
    outline_u_resolution: Option<WebGlUniformLocation>,
    outline_u_thickness: Option<WebGlUniformLocation>,
    outline_u_outline_color: Option<WebGlUniformLocation>,
    outline_u_object_color: Option<WebGlUniformLocation>,
    outline_u_background_color: Option<WebGlUniformLocation>,

    // Geometry
    quad_vao: VertexArray,
    quad_vbo: Buffer,

    // 3D Model Data
    model: GltfModel,

    // Framebuffers and Textures
    object_texture: Texture,      // Color target for 3D render
    object_depth_texture: Texture,// Depth target for 3D render
    object_fb: Framebuffer,       // FBO for 3D object rendering
    jfa_textures: [Texture; 2],   // Ping-pong textures for JFA
    jfa_fbs: [Framebuffer; 2],    // Ping-pong FBOs for JFA
}

#[wasm_bindgen]
pub struct WasmApp {
    gl: Rc<GlowContext>,
    canvas: HtmlCanvasElement,
    resources: Option<GlResources>, // Option<> allows for graceful failure during init
    width: i32,
    height: i32,
    last_time: f64,
    needs_resize: bool, // Flag to trigger resource recreation on resize
    // Camera parameters
    camera_pos: glm::Vec3,
    camera_target: glm::Vec3,
    camera_up: glm::Vec3,
    // Animation state
    model_rotation_y: f32,
}

// --- GLTF Loading Logic ---
fn load_gltf_resources(gl: &GlowContext) -> Result<GltfModel, AppError> {
    log("Loading GLTF model from embedded bytes...");
    let (doc, buffer_data_views, _images) = gltf::import_slice(MODEL_BYTES)
        .map_err(|e| AppError::GltfParse(format!("GLTF import error: {}", e)))?;

    // 1. Create WebGL buffers from glTF buffer views
    // We create one GL buffer per glTF buffer. Accessors will point into these.
    let mut gl_buffers = Vec::with_capacity(doc.buffers().count());
    unsafe {
        for buffer in doc.buffers() {
            // Find the corresponding raw data slice from buffer_data_views
            let raw_data = buffer_data_views.get(buffer.index())
                         .ok_or_else(|| AppError::GltfLogic(format!("Buffer data view missing for buffer index {}", buffer.index())))?;

            let gl_buffer = gl.create_buffer().map_err(|_| AppError::BufferCreationFailed)?;
            gl.bind_buffer(GL::ARRAY_BUFFER, Some(gl_buffer));
            gl.buffer_data_u8_slice(GL::ARRAY_BUFFER, raw_data, GL::STATIC_DRAW);
            gl_buffers.push(gl_buffer);
            log(&format!("Created GL buffer {} ({} bytes)", buffer.index(), raw_data.len()));
        }
        gl.bind_buffer(GL::ARRAY_BUFFER, None); // Unbind
    }
    log(&format!("Created {} WebGL buffers.", gl_buffers.len()));

    // 2. Process Meshes and Primitives: Create VAOs
    let mut meshes = Vec::with_capacity(doc.meshes().count());
    for mesh in doc.meshes() {
         let mesh_name = mesh.name().map(str::to_string);
         log(&format!("Processing mesh: {:?}", mesh_name));
         let mut primitives = Vec::with_capacity(mesh.primitives().count());

         for primitive in mesh.primitives() {
             log(&format!("  Processing primitive (mode: {:?})", primitive.mode()));
             let _reader = primitive.reader(|buffer| Some(&buffer_data_views[buffer.index()]));

             let vao = unsafe { gl.create_vertex_array().map_err(|_| AppError::VaoCreationFailed)? };
             unsafe { gl.bind_vertex_array(Some(vao)) };

             // --- Bind Attributes (Position mandatory, others optional) ---
             for (semantic, accessor) in primitive.attributes() {
                 let location = match semantic {
                     gltf::Semantic::Positions => 0,
                     // Add other attributes if needed by shaders
                     // gltf::Semantic::Normals => 1,
                     // gltf::Semantic::TexCoords(0) => 2,
                     _ => {
                         log(&format!("    Skipping unused attribute: {:?}", semantic));
                         continue; // Skip attributes not used by our current shaders
                     }
                 };

                 let buffer_view = accessor.view().ok_or_else(|| AppError::GltfLogic(format!("Accessor for {:?} has no view", semantic)))?;
                 let gl_buffer = gl_buffers[buffer_view.buffer().index()]; // Get the corresponding GL buffer

                 unsafe {
                     gl.bind_buffer(GL::ARRAY_BUFFER, Some(gl_buffer)); // Bind the correct buffer
                     gl.enable_vertex_attrib_array(location);
                     gl.vertex_attrib_pointer_f32(
                         location,                                      // Shader location
                         accessor.dimensions().multiplicity() as i32,   // Size (e.g., 3 for vec3)
                         accessor.data_type().as_gl_enum(),             // Type (e.g., GL::FLOAT)
                         accessor.normalized(),                         // Normalized?
                         buffer_view.stride().unwrap_or(0) as i32,      // Stride
                         accessor.offset() as i32 + buffer_view.offset() as i32, // Total offset = accessor offset + view offset
                     );
                 }
                 log(&format!("    Bound attribute {:?} to loc {} (buffer={}, offset={}, stride={})",
                     semantic, location, buffer_view.buffer().index(), accessor.offset() + buffer_view.offset(), buffer_view.stride().unwrap_or(0)));
             }


             // --- Bind Index Buffer (if present) ---
             let (index_count, index_type, has_indices) =
                 if let Some(indices_accessor) = primitive.indices() {
                     let buffer_view = indices_accessor.view().ok_or_else(|| AppError::GltfLogic("Indices accessor has no view".into()))?;
                     let gl_buffer = gl_buffers[buffer_view.buffer().index()];

                     unsafe {
                         // Bind the index buffer to the ELEMENT_ARRAY_BUFFER target *while the VAO is bound*
                         // This makes the EBO part of the VAO's state.
                         gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(gl_buffer));
                     }
                     log(&format!("    Bound index buffer (buffer={}, offset={}, count={})",
                         buffer_view.buffer().index(), indices_accessor.offset() + buffer_view.offset(), indices_accessor.count()));

                     (
                         indices_accessor.count() as i32,
                         indices_accessor.data_type().as_gl_enum(),
                         true,
                     )
                 } else {
                     // No indices, use draw_arrays. Count comes from position attribute.
                     let pos_accessor = primitive.get(&gltf::Semantic::Positions)
                                        .ok_or_else(|| AppError::GltfLogic("Primitive missing positions".into()))?;
                     log("    No index buffer found, will use draw_arrays.");
                     (pos_accessor.count() as i32, 0, false)
                 };

            // Store the processed primitive info
             primitives.push(Primitive {
                 vao,
                 index_count,
                 index_type,
                 mode: primitive.mode().as_gl_enum(),
                 has_indices,
             });

             // IMPORTANT: Unbind VAO after configuration
             unsafe { gl.bind_vertex_array(None) };
             // Unbind ARRAY_BUFFER too, just to be safe
             unsafe { gl.bind_buffer(GL::ARRAY_BUFFER, None) };
             // ELEMENT_ARRAY_BUFFER is unbound automatically when VAO is unbound
         }
         meshes.push(Mesh { primitives, name: mesh_name });
     }
    log(&format!("Processed {} meshes.", meshes.len()));


    // 3. Process Nodes (Hierarchy and Transforms)
    let mut nodes = Vec::with_capacity(doc.nodes().count());
    for node in doc.nodes() {
         let node_name = node.name().map(str::to_string);
        let (translation, rotation, scale) = node.transform().decomposed();

        let rotation_quat = glm::make_quat(&rotation);
        let transform = glm::translation(&glm::make_vec3(&translation))
                      * glm::quat_cast(&rotation_quat) // Use quat_cast
                      * glm::scaling(&glm::make_vec3(&scale));

        nodes.push(Node {
            mesh_index: node.mesh().map(|m| m.index()),
            transform,
            children: node.children().map(|c| c.index()).collect(),
            name: node_name,
        });
    }
     log(&format!("Processed {} nodes.", nodes.len()));

    // 4. Determine Root Nodes for the Default Scene
    let root_nodes = doc.scenes()
        .flat_map(|scene| scene.nodes()) // Usually only one scene in GLB
        .map(|node| node.index())
        .collect::<Vec<_>>();

    if root_nodes.is_empty() && !nodes.is_empty() {
        return Err(AppError::GltfLogic("GLTF has nodes but no scene/root nodes defined.".into()));
    }
     log(&format!("Found {} root nodes.", root_nodes.len()));


    Ok(GltfModel {
        meshes,
        nodes,
        root_nodes,
        gl_buffers, // Transfer ownership of GL buffers
    })
}


// --- WasmApp Implementation ---

#[wasm_bindgen]
impl WasmApp {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str, initial_width: i32, initial_height: i32) -> Result<WasmApp, JsValue> {
        // Wrap internal constructor with panic hook for better JS errors
        std::panic::set_hook(Box::new(console_error_panic_hook::hook)); // Set panic hook once

        let result = std::panic::catch_unwind(|| {
            Self::_new_internal(canvas_id, initial_width, initial_height)
        });

        match result {
            Ok(Ok(app)) => Ok(app),
            Ok(Err(e)) => {
                log(&format!("Error during WasmApp creation: {}", e));
                Err(JsValue::from_str(&format!("WasmApp creation failed: {}", e)))
            }
            Err(panic_payload) => {
                log("Panic during WasmApp creation!");
                let msg = match panic_payload.downcast_ref::<&'static str>() {
                    Some(s) => *s,
                    None => match panic_payload.downcast_ref::<String>() {
                        Some(s) => s.as_str(),
                        None => "Unknown panic payload",
                    },
                };
                Err(JsValue::from_str(&format!("WasmApp creation panicked: {}", msg)))
            }
        }
    }

     // Internal constructor returning Rust Result
    fn _new_internal(canvas_id: &str, initial_width: i32, initial_height: i32) -> Result<WasmApp, AppError> {
        log("Creating WasmApp...");
        let document = web_sys::window().ok_or(AppError::WebGlContext)?.document().ok_or(AppError::WebGlContext)?;
        let canvas = document
            .get_element_by_id(canvas_id)
            .ok_or(AppError::WebGlContext)?
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| AppError::WebGlContext)?;

        let options = serde_wasm_bindgen::to_value(&serde_json::json!({"antialias": false}))
            .map_err(|_| AppError::WebGlContext)?;

        // Get WebGL2 Context
        let webgl_context = canvas
            .get_context_with_context_options("webgl2", &options) // No AA needed for object buffer
            .map_err(|_| AppError::WebGlContext)?
            .ok_or(AppError::WebGlContext)?
            .dyn_into::<GL>()
            .map_err(|_| AppError::WebGlContext)?;

        // Check required extensions
        // if webgl_context.get_extension("EXT_color_buffer_float").map_err(|_| AppError::MissingExtension("EXT_color_buffer_float".to_string()))?.is_none() {
        //       return Err(AppError::MissingExtension("EXT_color_buffer_float is required.".to_string()));
        // }
        // if webgl_context.get_extension("WEBGL_depth_texture").map_err(|_| AppError::MissingExtension("WEBGL_depth_texture".to_string()))?.is_none() {
        //     return Err(AppError::MissingExtension("WEBGL_depth_texture is required for 3D.".to_string()));
        // }

        let gl = Rc::new(GlowContext::from_webgl2_context(webgl_context));

        // Initial camera setup
        let camera_pos = glm::vec3(0.0, 1.0, 3.0); // Position
        let camera_target = glm::vec3(0.0, 0.0, 0.0); // Look at origin
        let camera_up = glm::vec3(0.0, 1.0, 0.0); // Y is up

        let mut app = WasmApp {
            gl: gl.clone(),
            canvas,
            resources: None, // Resources will be created by resize_internal
            width: 0,
            height: 0,
            last_time: 0.0,
            needs_resize: true, // Force initial setup
            camera_pos,
            camera_target,
            camera_up,
            model_rotation_y: 0.0, // Initial rotation
        };

        // Perform initial resource setup
        app.resize_internal(initial_width, initial_height)?;

        log("WasmApp created successfully.");
        Ok(app)
    }

     // Setup or re-setup all GL resources
    fn setup_resources(&mut self) -> Result<(), AppError> {
        log(&format!("Setting up GL resources for {}x{}", self.width, self.height));
        let gl = &self.gl;

        // --- Cleanup old resources if they exist ---
        if let Some(old_resources) = self.resources.take() {
           self.cleanup_resources(old_resources);
        }

        // --- Load GLTF Model and create its GL resources ---
        let model = load_gltf_resources(gl)?;

        // --- Compile Shader Programs ---
        let object_program_3d = create_program(gl, OBJECT_VS, OBJECT_FS, Some(&[(0, "a_pos")]))?;
        let jfa_init_program = create_program(gl, FULLSCREEN_VS, JFA_INIT_FS, Some(&[(0, "a_pos")]))?;
        let jfa_step_program = create_program(gl, FULLSCREEN_VS, JFA_STEP_FS, Some(&[(0, "a_pos")]))?;
        let outline_program = create_program(gl, FULLSCREEN_VS, OUTLINE_FS, Some(&[(0, "a_pos")]))?;

        // --- Get Uniform Locations ---
        // (Using unsafe block for conciseness, ensure uniform names match shaders)
        let (
            object_u_projection, object_u_view, object_u_model,
            jfa_init_u_object_tex, jfa_init_u_resolution,
            jfa_step_u_jfa_tex, jfa_step_u_resolution, jfa_step_u_step_size,
            outline_u_object_tex, outline_u_jfa_tex, outline_u_resolution,
            outline_u_thickness, outline_u_outline_color, outline_u_object_color,
            outline_u_background_color
        ) = unsafe {(
            gl.get_uniform_location(object_program_3d, "u_projection"),
            gl.get_uniform_location(object_program_3d, "u_view"),
            gl.get_uniform_location(object_program_3d, "u_model"),
            gl.get_uniform_location(jfa_init_program, "u_object_texture"),
            gl.get_uniform_location(jfa_init_program, "u_resolution"),
            gl.get_uniform_location(jfa_step_program, "u_jfa_texture"),
            gl.get_uniform_location(jfa_step_program, "u_resolution"),
            gl.get_uniform_location(jfa_step_program, "u_step_size"),
            gl.get_uniform_location(outline_program, "u_object_texture"),
            gl.get_uniform_location(outline_program, "u_jfa_texture"),
            gl.get_uniform_location(outline_program, "u_resolution"),
            gl.get_uniform_location(outline_program, "u_outline_thickness"),
            gl.get_uniform_location(outline_program, "u_outline_color"),
            gl.get_uniform_location(outline_program, "u_object_color"),
            gl.get_uniform_location(outline_program, "u_background_color"),
        )};
        // Check that critical uniforms were found (optional but recommended)
        if object_u_projection.is_none() || object_u_view.is_none() || object_u_model.is_none() {
             return Err(AppError::ResourceCreation("Missing core 3D object uniforms".into()));
        }

        // --- Create Fullscreen Quad Geometry ---
        let quad_vao;
        let quad_vbo;
        let quad_vertices: [f32; 12] = [ -1.0, -1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 1.0, ];
        unsafe {
            quad_vao = gl.create_vertex_array().map_err(|_| AppError::VaoCreationFailed)?;
            quad_vbo = gl.create_buffer().map_err(|_| AppError::BufferCreationFailed)?;
            gl.bind_vertex_array(Some(quad_vao));
            gl.bind_buffer(GL::ARRAY_BUFFER, Some(quad_vbo));
            let u8_slice = core::slice::from_raw_parts( quad_vertices.as_ptr() as *const u8, quad_vertices.len() * core::mem::size_of::<f32>(), );
            gl.buffer_data_u8_slice(GL::ARRAY_BUFFER, u8_slice, GL::STATIC_DRAW);
            gl.enable_vertex_attrib_array(0); // location 0 for a_pos
            gl.vertex_attrib_pointer_f32(0, 2, GL::FLOAT, false, 0, 0);
            gl.bind_vertex_array(None);
            gl.bind_buffer(GL::ARRAY_BUFFER, None);
        }

        // --- Create Framebuffers and Textures ---
        // 1. Object Render Target (Color + Depth)
        let object_texture = create_render_target_texture(gl, self.width, self.height, GL::RGBA8, GL::RGBA, GL::UNSIGNED_BYTE)?;
        let object_depth_texture = create_render_target_texture(gl, self.width, self.height, GL::DEPTH_COMPONENT24, GL::DEPTH_COMPONENT, GL::UNSIGNED_INT)?;
        let object_fb = create_framebuffer(gl, object_texture, Some(object_depth_texture))?;

        // 2. JFA Ping-Pong Targets (Float Color)
        let jfa_texture1 = create_render_target_texture(gl, self.width, self.height, GL::RGBA32F, GL::RGBA, GL::FLOAT)?;
        let jfa_fb1 = create_framebuffer(gl, jfa_texture1, None)?; // No depth needed for JFA passes
        let jfa_texture2 = create_render_target_texture(gl, self.width, self.height, GL::RGBA32F, GL::RGBA, GL::FLOAT)?;
        let jfa_fb2 = create_framebuffer(gl, jfa_texture2, None)?;


        // --- Store all created resources ---
        self.resources = Some(GlResources {
            object_program_3d, jfa_init_program, jfa_step_program, outline_program,
            object_u_projection, object_u_view, object_u_model,
            jfa_init_u_object_tex, jfa_init_u_resolution,
            jfa_step_u_jfa_tex, jfa_step_u_resolution, jfa_step_u_step_size,
            outline_u_object_tex, outline_u_jfa_tex, outline_u_resolution,
            outline_u_thickness, outline_u_outline_color, outline_u_object_color,
            outline_u_background_color,
            quad_vao, quad_vbo,
            model,
            object_texture, object_depth_texture, object_fb,
            jfa_textures: [jfa_texture1, jfa_texture2],
            jfa_fbs: [jfa_fb1, jfa_fb2],
        });

        log("GL resources setup complete.");
        Ok(())
    }

    // Clean up GL resources associated with the GlResources struct
    fn cleanup_resources(&self, res: GlResources) {
         log("Cleaning up GL resources...");
         let gl = &self.gl;
         unsafe {
             // Programs
             gl.delete_program(res.object_program_3d);
             gl.delete_program(res.jfa_init_program);
             gl.delete_program(res.jfa_step_program);
             gl.delete_program(res.outline_program);

             // Quad geometry
             gl.delete_vertex_array(res.quad_vao);
             gl.delete_buffer(res.quad_vbo);

             // Model geometry (VAOs are in primitives, Buffers are stored separately)
             for mesh in res.model.meshes.iter() {
                 for primitive in mesh.primitives.iter() {
                     gl.delete_vertex_array(primitive.vao);
                 }
             }
             for buffer in res.model.gl_buffers.iter() {
                 gl.delete_buffer(*buffer);
             }

             // Textures
             gl.delete_texture(res.object_texture);
             gl.delete_texture(res.object_depth_texture);
             gl.delete_texture(res.jfa_textures[0]);
             gl.delete_texture(res.jfa_textures[1]);

             // Framebuffers
             gl.delete_framebuffer(res.object_fb);
             gl.delete_framebuffer(res.jfa_fbs[0]);
             gl.delete_framebuffer(res.jfa_fbs[1]);
         }
         log("GL resources cleanup complete.");
     }

    // Internal resize logic that recreates resources
    fn resize_internal(&mut self, width: i32, height: i32) -> Result<(), AppError> {
        log(&format!("Resizing canvas and resources to {}x{}", width, height));
        self.width = width;
        self.height = height;
        self.canvas.set_width(width as u32);
        self.canvas.set_height(height as u32);
        // Recreate all resources for the new size
        self.setup_resources()?;
        self.needs_resize = false; // Mark as resized
        Ok(())
    }

    // Public resize function called from JS
    #[wasm_bindgen]
    pub fn resize(&mut self, width: i32, height: i32) {
        if width == self.width && height == self.height {
            return; // No change
        }
        // Use internal resize and handle potential errors
        if let Err(e) = self.resize_internal(width, height) {
            log(&format!("Error during resize setup: {}", e));
            // If resizing failed, mark that we still need to resize on the next frame
            self.needs_resize = true;
        }
    }

    // --- Main Render Loop ---
    #[wasm_bindgen]
    pub fn render(&mut self, timestamp: f64) {

        let dt = (timestamp - self.last_time) / 1000.0; // Delta time in seconds
        self.last_time = timestamp;

        // --- Handle Resource Setup on Resize ---
        // If a previous resize failed, try again now.
        if self.needs_resize {
            if let Err(e) = self.setup_resources() {
                log(&format!("Error setting up resources in render loop: {}", e));
                // Don't attempt to render if setup failed
                return;
            }
            self.needs_resize = false;
        }

        // --- Get access to resources (should exist now) ---
        let res = match &self.resources {
            Some(r) => r,
            None => {
                log("Error: Render called but resources are not initialized.");
                return; // Should not happen if needs_resize logic is correct
            }
        };
        let gl = &self.gl;

        // --- Update Animation State ---
        self.model_rotation_y += (dt as f32) * 0.5; // Rotate model slowly

        // --- Camera and Projection Matrices ---
        let view_matrix = glm::look_at_rh(&self.camera_pos, &self.camera_target, &self.camera_up);
        let aspect_ratio = if self.height > 0 { self.width as f32 / self.height as f32 } else { 1.0 };
        let projection_matrix = glm::perspective_rh_zo(
            aspect_ratio,
            glm::radians(&glm::vec1(60.0)).x, // Field of view
            0.1,                             // Near plane
            100.0,                           // Far plane
        );
        let model_matrix = glm::rotate_y(&glm::identity(), self.model_rotation_y); // Apply rotation

        // --- Render Constants ---
         let outline_thickness = 5.0 + 3.0 * (timestamp / 1000.0 * 2.0).sin().abs(); // Animated thickness
         let outline_color = [0.0, 1.0, 0.0, 1.0]; // Green outline
         let object_fill_color = [0.8, 0.8, 0.8, 1.0]; // Grey fill for object in final render
         let background_color = [0.1, 0.1, 0.15, 1.0]; // Dark background

        // --- Render Passes ---
        unsafe {
            // === Pass 1: Render 3D Object Silhouette to object_fb ===
            gl.bind_framebuffer(GL::FRAMEBUFFER, Some(res.object_fb));
            gl.viewport(0, 0, self.width, self.height);
            gl.clear_color(0.0, 0.0, 0.0, 0.0); // Clear color to transparent black
            gl.clear_depth_f32(1.0);           // Clear depth buffer to max depth
            gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT); // Clear both

            gl.enable(GL::DEPTH_TEST); // Enable depth testing for 3D
            gl.depth_func(GL::LEQUAL); // Standard depth function
            // Optional: Enable face culling if model has consistent winding order
            // gl.enable(GL::CULL_FACE);
            // gl.cull_face(GL::BACK);

            gl.use_program(Some(res.object_program_3d));

            // Set camera uniforms
            gl.uniform_matrix_4_f32_slice(res.object_u_projection.as_ref(), false, projection_matrix.as_slice());
            gl.uniform_matrix_4_f32_slice(res.object_u_view.as_ref(), false, view_matrix.as_slice());

            // --- Recursive Node Rendering Function ---
            fn render_node_recursive(
                 gl: &GlowContext,
                 res: &GlResources, // Pass needed resources
                 node_idx: usize,
                 parent_transform: &glm::Mat4,
            ) {
                 if node_idx >= res.model.nodes.len() { return; } // Bounds check
                 let node = &res.model.nodes[node_idx];
                 let current_world_transform = parent_transform * node.transform;

                 // Render mesh if this node has one
                 if let Some(mesh_idx) = node.mesh_index {
                     if mesh_idx < res.model.meshes.len() { // Bounds check
                         let mesh = &res.model.meshes[mesh_idx];
                         unsafe {
                             // Set the model matrix for this specific mesh instance
                             gl.uniform_matrix_4_f32_slice(res.object_u_model.as_ref(), false, current_world_transform.as_slice());
                         }

                         for primitive in mesh.primitives.iter() {
                             unsafe {
                                 gl.bind_vertex_array(Some(primitive.vao)); // Bind the VAO for this primitive
                                 if primitive.has_indices {
                                     // Draw using indices (EBO is part of VAO state)
                                     gl.draw_elements(primitive.mode, primitive.index_count, primitive.index_type, 0); // Offset is 0 assuming EBO data starts at beginning
                                 } else {
                                     // Draw using vertex order
                                     gl.draw_arrays(primitive.mode, 0, primitive.index_count);
                                 }
                                 // No need to unbind VAO inside loop, unbind after mesh/node render if desired
                             }
                         }
                     } else { log(&format!("Warn: Node {:?} points to invalid mesh index {}", node.name, mesh_idx)); }
                 }

                 // Render children recursively
                 for child_idx in node.children.iter() {
                     render_node_recursive(gl, res, *child_idx, &current_world_transform);
                 }
            }

            // Start rendering from root nodes
            let root_transform = model_matrix; // Apply global model transform here
            for root_idx in res.model.root_nodes.iter() {
                render_node_recursive(gl, res, *root_idx, &root_transform);
            }

            // Clean up state after 3D pass
            gl.disable(GL::DEPTH_TEST);
            // gl.disable(GL::CULL_FACE);
            gl.bind_vertex_array(None); // Unbind any VAO left bound

            // === Pass 2: Initialize JFA Texture (jfa_fbs[0]) ===
            gl.bind_framebuffer(GL::FRAMEBUFFER, Some(res.jfa_fbs[0]));
            gl.viewport(0, 0, self.width, self.height); // Set viewport for this FBO
            // No clear needed - we overwrite every pixel

            gl.use_program(Some(res.jfa_init_program));
            gl.active_texture(GL::TEXTURE0);
            gl.bind_texture(GL::TEXTURE_2D, Some(res.object_texture)); // Use the silhouette texture
            gl.uniform_1_i32(res.jfa_init_u_object_tex.as_ref(), 0); // Texture unit 0
            gl.uniform_2_f32(res.jfa_init_u_resolution.as_ref(), self.width as f32, self.height as f32);

            // Draw fullscreen quad
            gl.bind_vertex_array(Some(res.quad_vao));
            gl.draw_arrays(GL::TRIANGLES, 0, 6);
            gl.bind_vertex_array(None);
            gl.bind_texture(GL::TEXTURE_2D, None); // Unbind texture

            // === Pass 3: JFA Steps (Ping-Pong between jfa_fbs[0] and jfa_fbs[1]) ===
             let num_passes = (self.width.max(self.height) as f32).log2().ceil() as i32;
             let mut read_index = 0; // Start reading from texture 0 (where init data is)
             let mut write_index = 1; // Start writing to texture 1

             gl.use_program(Some(res.jfa_step_program)); // Use JFA step program for all passes
             gl.bind_vertex_array(Some(res.quad_vao)); // Use quad for all passes

             for i in 0..num_passes {
                  // Calculate step size (jump distance in pixels) for this pass
                 let step_size = ( (self.width.max(self.height) as f32) / 2.0f32.powi(i + 1) ).max(1.0);

                 // Bind the FBO to write to
                 gl.bind_framebuffer(GL::FRAMEBUFFER, Some(res.jfa_fbs[write_index]));
                 gl.viewport(0, 0, self.width, self.height); // Ensure viewport is set for the FBO

                 // Bind the texture to read from (result of previous pass)
                 gl.active_texture(GL::TEXTURE0);
                 gl.bind_texture(GL::TEXTURE_2D, Some(res.jfa_textures[read_index]));

                 // Set uniforms for this pass
                 gl.uniform_1_i32(res.jfa_step_u_jfa_tex.as_ref(), 0); // Texture unit 0
                 gl.uniform_2_f32(res.jfa_step_u_resolution.as_ref(), self.width as f32, self.height as f32);
                 gl.uniform_1_f32(res.jfa_step_u_step_size.as_ref(), step_size);

                 // Draw fullscreen quad to perform the JFA step
                 gl.draw_arrays(GL::TRIANGLES, 0, 6);

                 // Swap read/write indices for the next pass
                 std::mem::swap(&mut read_index, &mut write_index);
             }
             // After the loop, the final JFA result is in jfa_textures[read_index]

             let final_jfa_texture = res.jfa_textures[read_index];
             gl.bind_vertex_array(None); // Unbind quad VAO
             gl.active_texture(GL::TEXTURE0);
             gl.bind_texture(GL::TEXTURE_2D, None); // Unbind texture


            // === Pass 4: Render Final Composite to Screen ===
            gl.bind_framebuffer(GL::FRAMEBUFFER, None); // Bind default framebuffer (screen)
            gl.viewport(0, 0, self.width, self.height); // Set viewport to canvas size
            gl.clear_color(background_color[0], background_color[1], background_color[2], background_color[3]);
            gl.clear(GL::COLOR_BUFFER_BIT); // Only clear color buffer

            gl.use_program(Some(res.outline_program));

            // Bind textures: Object silhouette and final JFA result
            gl.active_texture(GL::TEXTURE0);
            gl.bind_texture(GL::TEXTURE_2D, Some(res.object_texture));
            gl.uniform_1_i32(res.outline_u_object_tex.as_ref(), 0); // Unit 0

            gl.active_texture(GL::TEXTURE1);
            gl.bind_texture(GL::TEXTURE_2D, Some(final_jfa_texture));
            gl.uniform_1_i32(res.outline_u_jfa_tex.as_ref(), 1); // Unit 1

            // Set other outline uniforms
            gl.uniform_2_f32(res.outline_u_resolution.as_ref(), self.width as f32, self.height as f32);
            gl.uniform_1_f32(res.outline_u_thickness.as_ref(), outline_thickness as f32);
            gl.uniform_4_f32(res.outline_u_outline_color.as_ref(), outline_color[0], outline_color[1], outline_color[2], outline_color[3]);
            gl.uniform_4_f32(res.outline_u_object_color.as_ref(), object_fill_color[0], object_fill_color[1], object_fill_color[2], object_fill_color[3]);
            gl.uniform_4_f32(res.outline_u_background_color.as_ref(), background_color[0], background_color[1], background_color[2], background_color[3]);

            // Draw fullscreen quad for compositing
            gl.bind_vertex_array(Some(res.quad_vao));
            gl.draw_arrays(GL::TRIANGLES, 0, 6);
            gl.bind_vertex_array(None);

            // Clean up texture units
            gl.active_texture(GL::TEXTURE1);
            gl.bind_texture(GL::TEXTURE_2D, None);
            gl.active_texture(GL::TEXTURE0);
            gl.bind_texture(GL::TEXTURE_2D, None);

            gl.flush();
        }
    }
}

// --- Cleanup on Drop ---
impl Drop for WasmApp {
     fn drop(&mut self) {
         log("Dropping WasmApp and cleaning resources...");
         if let Some(res) = self.resources.take() {
            let gl = &self.gl;
            unsafe {
                 // Programs
                 gl.delete_program(res.object_program_3d);
                 gl.delete_program(res.jfa_init_program);
                 gl.delete_program(res.jfa_step_program);
                 gl.delete_program(res.outline_program);

                 // Quad geometry
                 gl.delete_vertex_array(res.quad_vao);
                 gl.delete_buffer(res.quad_vbo);

                 // Model geometry
                 for mesh in res.model.meshes.iter() {
                     for primitive in mesh.primitives.iter() {
                         gl.delete_vertex_array(primitive.vao);
                     }
                 }
                 for buffer in res.model.gl_buffers.iter() {
                     gl.delete_buffer(*buffer);
                 }

                 // Textures
                 gl.delete_texture(res.object_texture);
                 gl.delete_texture(res.object_depth_texture);
                 gl.delete_texture(res.jfa_textures[0]);
                 gl.delete_texture(res.jfa_textures[1]);

                 // Framebuffers
                 gl.delete_framebuffer(res.object_fb);
                 gl.delete_framebuffer(res.jfa_fbs[0]);
                 gl.delete_framebuffer(res.jfa_fbs[1]);
            }
         }
         log("WasmApp cleanup finished.");
     }
}

// --- Wasm Entry Point ---
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    log("Wasm module loaded and started.");
    Ok(())
}