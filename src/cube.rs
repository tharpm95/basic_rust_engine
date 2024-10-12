use glium::implement_vertex;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

#[derive(Copy, Clone)]
pub struct Instance {
    pub instance_position: [f32; 3],
}

implement_vertex!(Instance, instance_position);

pub struct Cube {
    pub position: [f32; 3],
}

impl Cube {
    pub fn vertices() -> Vec<Vertex> {
        vec![
            // Front face
            Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 0.0] },
            Vertex { position: [ 0.5, -0.5, -0.5], tex_coords: [1.0, 0.0] },
            Vertex { position: [ 0.5,  0.5, -0.5], tex_coords: [1.0, 1.0] },
            Vertex { position: [-0.5,  0.5, -0.5], tex_coords: [0.0, 1.0] },

            // Back face
            Vertex { position: [-0.5, -0.5,  0.5], tex_coords: [0.0, 0.0] },
            Vertex { position: [ 0.5, -0.5,  0.5], tex_coords: [1.0, 0.0] },
            Vertex { position: [ 0.5,  0.5,  0.5], tex_coords: [1.0, 1.0] },
            Vertex { position: [-0.5,  0.5,  0.5], tex_coords: [0.0, 1.0] },

            // Top face
            Vertex { position: [-0.5,  0.5, -0.5], tex_coords: [0.0, 0.0] },
            Vertex { position: [ 0.5,  0.5, -0.5], tex_coords: [1.0, 0.0] },
            Vertex { position: [ 0.5,  0.5,  0.5], tex_coords: [1.0, 1.0] },
            Vertex { position: [-0.5,  0.5,  0.5], tex_coords: [0.0, 1.0] },

            // Bottom face
            Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 0.0] },
            Vertex { position: [ 0.5, -0.5, -0.5], tex_coords: [1.0, 0.0] },
            Vertex { position: [ 0.5, -0.5,  0.5], tex_coords: [1.0, 1.0] },
            Vertex { position: [-0.5, -0.5,  0.5], tex_coords: [0.0, 1.0] },

            // Right face
            Vertex { position: [ 0.5, -0.5, -0.5], tex_coords: [0.0, 0.0] },
            Vertex { position: [ 0.5,  0.5, -0.5], tex_coords: [1.0, 0.0] },
            Vertex { position: [ 0.5,  0.5,  0.5], tex_coords: [1.0, 1.0] },
            Vertex { position: [ 0.5, -0.5,  0.5], tex_coords: [0.0, 1.0] },

            // Left face
            Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 0.0] },
            Vertex { position: [-0.5,  0.5, -0.5], tex_coords: [1.0, 0.0] },
            Vertex { position: [-0.5,  0.5,  0.5], tex_coords: [1.0, 1.0] },
            Vertex { position: [-0.5, -0.5,  0.5], tex_coords: [0.0, 1.0] },
        ]
    }

    pub fn indices() -> Vec<u16> {
        vec![
            0, 1, 2, 2, 3, 0,    // Front face
            4, 5, 6, 6, 7, 4,    // Back face
            8, 9, 10, 10, 11, 8, // Top face
            12, 13, 14, 14, 15, 12, // Bottom face
            16, 17, 18, 18, 19, 16, // Right face
            20, 21, 22, 22, 23, 20  // Left face
        ]
    }
}