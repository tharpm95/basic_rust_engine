use crate::camera::Camera;
use crate::world::World;

pub fn update_world(camera: &Camera, world: &mut World) {
    let current_chunk_pos = (
        (camera.eye.x / (world.chunk_size as f32)).floor() as i32,
        (camera.eye.z / (world.chunk_size as f32)).floor() as i32,
    );

    for dx in -1..=1 {
        for dz in -1..=1 {
            world.load_chunk((current_chunk_pos.0 + dx, current_chunk_pos.1 + dz));
        }
    }
}
