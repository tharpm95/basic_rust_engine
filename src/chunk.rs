use crate::vertex::{Vertex, VERTICES, INDICES};

pub fn generate_chunk_vertices(chunk_pos: (i32, i32), chunk_size: usize) -> Vec<Vertex> {
    let mut vertices = Vec::new();
    for x in 0..chunk_size {
        for z in 0..chunk_size {
            let base_position = [
                x as f32 + (chunk_pos.0 * chunk_size as i32) as f32,
                0.0,
                z as f32 + (chunk_pos.1 * chunk_size as i32) as f32,
            ];
            for vertex in VERTICES.iter() {
                let mut position = vertex.position;
                position[0] += base_position[0];
                position[2] += base_position[2];
                vertices.push(Vertex {
                    position,
                    tex_coords: vertex.tex_coords,
                });
            }
        }
    }
    vertices
}

pub fn generate_chunk_indices(chunk_size: usize) -> Vec<u16> {
    let mut indices = Vec::new();
    let vertex_count = INDICES.len() as u16;
    for x in 0..chunk_size {
        for z in 0..chunk_size {
            let offset = ((x * chunk_size + z) * vertex_count as usize) as u16;
            indices.extend(INDICES.iter().map(|i| i + offset));
        }
    }
    indices
}
