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
            Vertex { position: [0.5, -0.5, -0.5], tex_coords: [1.0, 0.0] },
            Vertex { position: [0.5, 0.5, -0.5], tex_coords: [1.0, 1.0] },
            Vertex { position: [-0.5, 0.5, -0.5], tex_coords: [0.0, 1.0] },

            // Back face
            Vertex { position: [-0.5, -0.5, 0.5], tex_coords: [0.0, 0.0] },
            Vertex { position: [0.5, -0.5, 0.5], tex_coords: [1.0, 0.0] },
            Vertex { position: [0.5, 0.5, 0.5], tex_coords: [1.0, 1.0] },
            Vertex { position: [-0.5, 0.5, 0.5], tex_coords: [0.0, 1.0] },
        ]
    }

    pub fn indices() -> Vec<u16> {
        vec![
            0, 1, 2, 2, 3, 0,    // Front face
            4, 5, 6, 6, 7, 4,    // Back face
            0, 1, 5, 5, 4, 0,    // Bottom face
            2, 3, 7, 7, 6, 2,    // Top face
            1, 5, 6, 6, 2, 1,    // Right face
            0, 4, 7, 7, 3, 0     // Left face
        ]
    }
}