use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

pub const VERTICES: &[Vertex] = &[
// Front face
Vertex { position: [-1.0, -1.0,  1.0], tex_coords: [0.0, 0.0] },
Vertex { position: [ 1.0, -1.0,  1.0], tex_coords: [1.0, 0.0] },
Vertex { position: [ 1.0,  1.0,  1.0], tex_coords: [1.0, 1.0] },
Vertex { position: [-1.0,  1.0,  1.0], tex_coords: [0.0, 1.0] },

// Back face
Vertex { position: [-1.0, -1.0, -1.0], tex_coords: [0.0, 0.0] },
Vertex { position: [-1.0,  1.0, -1.0], tex_coords: [0.0, 1.0] },
Vertex { position: [ 1.0,  1.0, -1.0], tex_coords: [1.0, 1.0] },
Vertex { position: [ 1.0, -1.0, -1.0], tex_coords: [1.0, 0.0] },

// Top face
Vertex { position: [-1.0, 1.0, -1.0], tex_coords: [0.0, 1.0] },
Vertex { position: [-1.0, 1.0,  1.0], tex_coords: [0.0, 0.0] },
Vertex { position: [ 1.0, 1.0,  1.0], tex_coords: [1.0, 0.0] },
Vertex { position: [ 1.0, 1.0, -1.0], tex_coords: [1.0, 1.0] },

// Bottom face
Vertex { position: [-1.0, -1.0, -1.0], tex_coords: [0.0, 0.0] },
Vertex { position: [ 1.0, -1.0, -1.0], tex_coords: [1.0, 0.0] },
Vertex { position: [ 1.0, -1.0,  1.0], tex_coords: [1.0, 1.0] },
Vertex { position: [-1.0, -1.0,  1.0], tex_coords: [0.0, 1.0] },

// Right face
Vertex { position: [ 1.0, -1.0, -1.0], tex_coords: [0.0, 0.0] },
Vertex { position: [ 1.0,  1.0, -1.0], tex_coords: [0.0, 1.0] },
Vertex { position: [ 1.0,  1.0,  1.0], tex_coords: [1.0, 1.0] },
Vertex { position: [ 1.0, -1.0,  1.0], tex_coords: [1.0, 0.0] },

// Left face
Vertex { position: [-1.0, -1.0, -1.0], tex_coords: [1.0, 0.0] },
Vertex { position: [-1.0, -1.0,  1.0], tex_coords: [1.0, 1.0] },
Vertex { position: [-1.0,  1.0,  1.0], tex_coords: [0.0, 1.0] },
Vertex { position: [-1.0,  1.0, -1.0], tex_coords: [0.0, 0.0] },
];

pub const INDICES: &[u16] = &[
    0, 1, 2, 2, 3, 0, // front
    4, 5, 6, 6, 7, 4, // back
    8, 9, 10, 10, 11, 8, // top
    12, 13, 14, 14, 15, 12, // bottom
    16, 17, 18, 18, 19, 16, // right
    20, 21, 22, 22, 23, 20, // left
];