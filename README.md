# Project Overview

Basic Rust Engine - Some basic components needed to build a graphics engine in Rust

## Getting Started

To run the project, ensure you have Rust installed and execute the following command in the project directory:

```bash
cargo run
```

## File Descriptions

- **`src/app.rs`**: Sets up and runs the main application loop. It initializes the graphics pipeline using `wgpu`, loads shaders and textures, and manages the camera and world state. The function `run` is the main entry point for the application logic. Here's a detailed summary of its functionality:
  - **Imports and Dependencies**: The file imports several modules and dependencies, including `wgpu` for graphics rendering, `winit` for window and event loop management, and various custom modules like `camera`, `world`, `vertex`, `uniforms`, `world_update`, `texture`, and `event_loop`.
  - **`run` Function**: This is the main asynchronous function that initializes the application. It sets up the graphics pipeline using `wgpu`, including creating an instance, surface, adapter, device, and queue. It configures the swapchain for rendering.
  - **Shader and Texture**: The function loads a shader module from `shader.wgsl` and a texture from an image file. These are used in the rendering process.
  - **Uniforms and Bind Groups**: It creates a uniform buffer and bind group layout, which are used to pass data to the GPU for rendering. This includes transformation matrices and textures.
  - **Depth Texture**: A depth texture is created for handling depth information in 3D rendering.
  - **Render Pipeline**: The function sets up a render pipeline, which defines how vertices and fragments are processed and rendered.
  - **Camera and World**: A `Camera` object is created to manage the view perspective, and a `World` object is initialized to manage the game world or environment. The `update_world` function is called to update the world state.
  - **Dynamic Buffers**: Dynamic vertex and index buffers are created to store vertex and index data for rendering.
  - **Concurrency**: Several components are wrapped in `Arc` and `Mutex` to allow for safe concurrent access, as they will be shared across threads in the event loop.
  - **Event Loop**: The `handle_event_loop` function is called to start the event loop, passing all necessary components. This loop handles user input and updates the application state.

- **`src/camera.rs`**: Defines the `Camera` struct and methods for managing the camera's position and orientation in 3D space. It includes methods for processing mouse movement and moving the camera in various directions.
  - **Camera Struct**: The `Camera` struct contains fields for the camera's position (`eye`), the point it is looking at (`target`), the up direction (`up`), and various parameters for perspective projection such as field of view (`fovy`), aspect ratio (`aspect`), near and far clipping planes (`znear`, `zfar`), and orientation angles (`yaw`, `pitch`).
  - **`new` Method**: Initializes a new `Camera` instance with default values, setting the camera's position, target, and orientation.
  - **`update_camera_vectors` Method**: Updates the camera's target vector based on its yaw and pitch angles, ensuring the camera is oriented correctly.
  - **`process_mouse_movement` Method**: Adjusts the camera's yaw and pitch based on mouse movement, applying a sensitivity factor. It also clamps the pitch to prevent gimbal lock and updates the camera vectors accordingly.
  - **`move_forward` Method**: Moves the camera forward along its viewing direction by a specified amount.
  - **`strafe_right` Method**: Moves the camera sideways (right) relative to its current orientation by a specified amount.
  - **`move_up` Method**: Moves the camera upward along the y-axis by a specified amount.

- **`src/chunk.rs`**: Provides functions for generating vertices and indices for chunks, which are segments of the game world. These functions are used to create the geometry of the chunks.
  - **Imports**: The file imports `Vertex`, `VERTICES`, and `INDICES` from the `vertex` module. These are used to define the geometry of the chunks.
  - **`generate_chunk_vertices` Function**: This function generates a vector of `Vertex` objects for a chunk. It takes the chunk's position (`chunk_pos`) and size (`chunk_size`) as parameters. The function iterates over the chunk's grid, adjusting the base position for each vertex and adding it to the list of vertices. This allows for the creation of a grid of vertices that represent the chunk in 3D space.
  - **`generate_chunk_indices` Function**: This function generates a vector of indices for a chunk. It takes the chunk size as a parameter and iterates over the grid, calculating the offset for each set of indices based on the chunk's position. The indices are used to define the order in which vertices are connected to form triangles, which are the basic building blocks of 3D models.

- **`src/event_loop.rs`**: Manages the application's event loop, handling user input and rendering updates. It processes window events, keyboard input, and mouse movement, and updates the camera and world state accordingly.
  - **Imports**: The file imports necessary modules for event handling, synchronization, and graphics rendering. It uses `winit` for event management and `wgpu` for graphics operations.
  - **`handle_event_loop` Function**: This function sets up and runs the event loop, which processes events such as window resizing, keyboard input, and mouse movement. It takes numerous parameters, including the event loop, window, and various graphics and application state components.
  - **Window Events**: The function handles window events, such as resizing and closing. When the window is resized, it updates the camera's aspect ratio and reconfigures the surface.
  - **Keyboard Input**: It tracks pressed keys using a `HashSet`, allowing for continuous input handling. This is used to move the camera based on key presses (`W`, `A`, `S`, `D` for movement, `Space` and `LShift` for vertical movement).
  - **Mouse Movement**: The function processes mouse movement to adjust the camera's orientation, using a sensitivity factor to control the rate of change.
  - **Redraw Requests**: On redraw requests, the function updates the camera and world state, recalculates vertex and index buffers if necessary, and submits rendering commands to the GPU.
  - **Rendering**: It creates a render pass, sets the pipeline and bind groups, and draws indexed vertices to render the scene.

- **`src/main.rs`**: The entry point of the application. It initializes the event loop and window, sets the window to fullscreen, and starts the main application logic by calling `app::run`.
  - **Imports**: The file imports necessary components from the `winit` crate for creating an event loop and window.
  - **Module Declarations**: It declares several modules, including `app`, `camera`, `world`, `vertex`, `uniforms`, `chunk`, `world_update`, `texture`, and `event_loop`. These modules contain the core functionality of the application.
  - **`main` Function**: This function initializes the application by creating an event loop and a window. It sets the window to fullscreen mode and attempts to grab the cursor, making it invisible for a more immersive experience.
  - **Running the Application**: The `pollster::block_on` function is used to run the asynchronous `app::run` function, passing the event loop and window as arguments. This starts the main application logic, including rendering and event handling.

- **`src/texture.rs`**: Handles texture creation and management. It defines the `Texture` struct and a method for creating a texture from an image file, which is used in the rendering pipeline.
  - **Texture Struct**: The `Texture` struct contains fields for a `wgpu::Texture`, `wgpu::TextureView`, and `wgpu::Sampler`. These components are essential for using textures in rendering.
  - **`from_image` Method**: This method creates a `Texture` from an image file. It takes a `wgpu::Device`, `wgpu::Queue`, and a file path as parameters. The method performs the following steps:
    - Opens the image file and converts it to RGBA format.
    - Retrieves the image dimensions and creates a `wgpu::Texture` with the appropriate size and format.
    - Writes the image data to the texture using the queue.
    - Creates a `TextureView` and a `Sampler` for the texture, which are used in the rendering pipeline to access and sample the texture.

- **`src/uniforms.rs`**: Defines the `Uniforms` struct and methods for managing transformation matrices. These matrices are used to transform 3D coordinates to 2D screen space.
  - **Uniforms Struct**: The `Uniforms` struct contains two fields: `view_proj` and `model`, both of which are 4x4 matrices. These matrices are used to transform 3D coordinates to 2D screen space.
  - **`new` Method**: Initializes a new `Uniforms` instance with identity matrices for both `view_proj` and `model`. Identity matrices are used as a starting point for transformations.
  - **`update_model` Method**: Updates the model matrix to apply a rotation around the y-axis. This is used to rotate objects in the scene.
  - **`update_view_proj` Method**: Updates the `view_proj` matrix based on the camera's position and orientation. It calculates the view matrix using the camera's eye, target, and up vectors, and the projection matrix using the camera's field of view, aspect ratio, and clipping planes. The combined view-projection matrix is used to transform world coordinates to screen coordinates.

- **`src/vertex.rs`**: Defines the `Vertex` struct and provides constants for vertex and index data. These are used to define the geometry of 3D models.
  - **Vertex Struct**: The `Vertex` struct contains fields for position and `tex_coords`. The position is a 3D coordinate, and `tex_coords` are 2D texture coordinates. The struct is marked with `#[repr(C)]` to ensure it has a C-compatible memory layout, and it derives `Copy`, `Clone`, `Pod`, and `Zeroable` traits for efficient data handling.
  - **`VERTICES` Constant**: This constant defines an array of `Vertex` instances representing the vertices of a cube. Each face of the cube is defined by four vertices, with associated texture coordinates.
  - **`INDICES` Constant**: This constant defines an array of indices that specify the order in which vertices are connected to form triangles. Each face of the cube is represented by two triangles, defined by six indices.

- **`src/world.rs`**: Defines the `Chunk` and `World` structs, which manage the game's world or environment. It includes methods for loading chunks and managing their geometry.
  - **Chunk Struct**: The `Chunk` struct contains vertices and indices, which are vectors of `Vertex` and `u16` respectively. These represent the geometry of a chunk, a segment of the game world.
  - **World Struct**: The `World` struct contains a `HashMap` of chunks, indexed by their position (`i32`, `i32`), and a `chunk_size` that defines the size of each chunk.
  - **`new` Method**: Initializes a new `World` instance with an empty `HashMap` for chunks and a specified `chunk_size`.
  - **`load_chunk` Method**: Loads a chunk at a given position if it is not already present in the chunks map. It generates the vertices and indices for the chunk using the `generate_chunk_vertices` and `generate_chunk_indices` functions from the `chunk` module and inserts the new chunk into the map.

- **`src/world_update.rs`**: Contains the `update_world` function, which updates the state of the game world based on the camera's position, ensuring that the necessary chunks are loaded.
  - **`update_world` Function**: This function takes a reference to a `Camera` and a mutable reference to a `World`. It calculates the current chunk position based on the camera's eye position and the world's `chunk_size`.
  - **Chunk Loading**: The function iterates over a 3x3 grid centered around the current chunk position, calling `world.load_chunk` for each position. This ensures that the chunks surrounding the camera's current position are loaded, allowing for seamless exploration of the game world.

## Additional Resources

- **`src/shader.wgsl`**: Contains shader code used for rendering. Shaders are programs that run on the GPU to control the rendering of graphics.

- **`src/images/`**: Directory containing image assets used in the project. These images are used as textures or other visual elements in the application.
