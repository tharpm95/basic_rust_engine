use cgmath::{Point3, Vector3, InnerSpace};

pub struct Camera {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub fovy: f32,
    pub aspect: f32,
    pub znear: f32,
    pub zfar: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            eye: (0.0, 1.0, 2.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: Vector3::unit_y(),
            fovy: 45.0f32.to_radians(),
            aspect: 16.0 / 9.0,
            znear: 0.1,
            zfar: 100.0,
            yaw: -90.0f32.to_radians(),
            pitch: 0.0,
        }
    }

    pub fn update_camera_vectors(&mut self) {
        let front = Vector3 {
            x: self.yaw.cos() * self.pitch.cos(),
            y: self.pitch.sin(),
            z: self.yaw.sin() * self.pitch.cos(),
        };
        self.target = self.eye + front.normalize();
    }

    pub fn process_mouse_movement(&mut self, xoffset: f32, yoffset: f32, sensitivity: f32) {
        self.yaw += xoffset * sensitivity;
        self.pitch += yoffset * sensitivity;

        if self.pitch > 89.0f32.to_radians() {
            self.pitch = 89.0f32.to_radians();
        }
        if self.pitch < -89.0f32.to_radians() {
            self.pitch = -89.0f32.to_radians();
        }

        self.update_camera_vectors();
    }

    pub fn move_forward(&mut self, amount: f32) {
        let forward = (self.target - self.eye).normalize();
        self.eye += forward * amount;
        self.target += forward * amount;
    }

    pub fn strafe_right(&mut self, amount: f32) {
        let forward = (self.target - self.eye).normalize();
        let right = forward.cross(self.up).normalize();
        self.eye += right * amount;
        self.target += right * amount;
    }

    pub fn move_up(&mut self, amount: f32) {
        let up = Vector3::unit_y();
        self.eye += up * amount;
        self.target += up * amount;
    }
}
