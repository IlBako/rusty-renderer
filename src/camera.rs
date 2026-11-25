use bytemuck::{Pod, Zeroable};
use glam::{Quat, Vec3};

pub struct Camera {
    target: Vec3,
    distance: f32,
    initial_distance: f32,
    fov: f32,
    orientation: Quat,
}

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct CameraUniform {
    position: [f32; 3],
    fov: f32,
    right: [f32; 4],
    up: [f32; 4],
    forward: [f32; 4],
}

const _: () = assert!(size_of::<CameraUniform>() == 64);

const ORBIT_SENSITIVITY: f32 = 0.005;
const PAN_SPEED: f32 = 0.001;

impl Camera {
    pub(crate) fn new(init_target: Vec3, init_distance: f32, fov: f32) -> Self {
        let orientation = Quat::IDENTITY;

        Self {
            target: init_target,
            distance: init_distance,
            initial_distance: init_distance,
            fov,
            orientation,
        }
    }

    pub(crate) fn to_uniform(&self) -> CameraUniform {
        let (right, up, forward) = self.compute_axis();

        let padded_right = right.extend(0.0).to_array();
        let padded_up = up.extend(0.0).to_array();
        let padded_forward = forward.extend(0.0).to_array();

        let position = (self.target - self.distance * forward).to_array();

        CameraUniform {
            position,
            right: padded_right,
            up: padded_up,
            forward: padded_forward,
            fov: self.fov,
        }
    }

    pub(crate) fn pan(&mut self, dx: f64, dy: f64) {
        let (right, up, _) = self.compute_axis();

        // Scale by distance so pan speed feels consistent at any zoom level
        let pan_speed = self.distance * PAN_SPEED;

        self.target -= right * (dx as f32) * pan_speed;
        self.target += up * (dy as f32) * pan_speed;
    }

    pub fn orbit(&mut self, dx: f64, dy: f64) {
        // 1. Rotate around fixed World Up (keeps the horizon level)
        let yaw_rot = Quat::from_axis_angle(Vec3::Y, -(dx as f32) * ORBIT_SENSITIVITY);

        // 2. Rotate around the camera's current local Right axis (allows infinite 360° pitch)
        let local_right = self.orientation * Vec3::X;
        let pitch_rot = Quat::from_axis_angle(local_right, -(dy as f32) * ORBIT_SENSITIVITY);

        // Combine and normalize to prevent floating-point drift
        self.orientation = (yaw_rot * pitch_rot * self.orientation).normalize();
    }

    pub fn dolly(&mut self, amount: f32) {
        // 0.9^1 = 0.9 (zoom in 10% per notch), 0.9^-1 ≈ 1.111 (zoom out ~11% per notch)
        self.distance *= 0.9_f32.powf(amount);

        // Prevent clipping inside the target (and optionally cap max distance)
        self.distance = self.distance.clamp(0.1, 1000.0);
    }

    pub fn reset_camera(&mut self) {
        self.orientation = Quat::IDENTITY;
        self.distance = self.initial_distance;
    }

    pub fn compute_axis(&self) -> (Vec3, Vec3, Vec3) {
        let right = self.orientation * Vec3::X;
        let up = self.orientation * Vec3::Y;
        let forward = self.orientation * -Vec3::Z;

        (right, up, forward)
    }
}
