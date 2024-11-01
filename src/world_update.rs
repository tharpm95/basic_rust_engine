use crate::camera::Camera;
use crate::world::World;

pub fn update_world(camera: &Camera, world: &mut World) {
    let current_chunk_pos = (
        (camera.eye.x / (world.chunk_size as f32)).floor() as i32,
        (camera.eye.z / (world.chunk_size as f32)).floor() as i32,
    );

    // Load chunks in a 3x3 area around the current chunk position
    for dx in -1..=1 {
        for dz in -1..=1 {
            world.load_chunk((current_chunk_pos.0 + dx, current_chunk_pos.1 + dz));
        }
    }

    // Unload chunks that are outside the view distance (optional logic)
    // This logic can be adjusted based on the desired view distance
    let unload_distance = 2; // Example distance to unload chunks
    let keys_to_remove: Vec<(i32, i32)> = world.chunks.keys()
        .filter(|&(pos)| {
            (pos.0 < current_chunk_pos.0 - unload_distance || pos.0 > current_chunk_pos.0 + unload_distance) ||
            (pos.1 < current_chunk_pos.1 - unload_distance || pos.1 > current_chunk_pos.1 + unload_distance)
        })
        .cloned()
        .collect();

    for key in keys_to_remove {
        world.chunks.remove(&key);
    }
}
