use std::collections::HashMap;
use crate::vertex::Vertex;
use crate::chunk::{generate_chunk_vertices, generate_chunk_indices};

pub struct Chunk {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
}

pub struct World {
    pub chunks: HashMap<(i32, i32), Chunk>,
    pub chunk_size: usize,
}

impl World {
    pub fn new(chunk_size: usize) -> Self {
        Self {
            chunks: HashMap::new(),
            chunk_size,
        }
    }

    pub fn load_chunk(&mut self, chunk_pos: (i32, i32)) {
        if !self.chunks.contains_key(&chunk_pos) {
            let vertices = generate_chunk_vertices(chunk_pos, self.chunk_size);
            let indices = generate_chunk_indices(self.chunk_size);

            self.chunks.insert(chunk_pos, Chunk {
                vertices,
                indices,
            });
        }
    }
}
