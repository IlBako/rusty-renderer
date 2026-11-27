use std::time::{SystemTime, UNIX_EPOCH};

use glam::Vec3;

pub(crate) trait Vec3Bounds {
    fn into_vec3_bounds(self) -> (Vec3, Vec3);
}

impl Vec3Bounds for (f32, f32) {
    fn into_vec3_bounds(self) -> (Vec3, Vec3) {
        (Vec3::splat(self.0), Vec3::splat(self.1))
    }
}

impl Vec3Bounds for (Vec3, Vec3) {
    fn into_vec3_bounds(self) -> (Vec3, Vec3) {
        self
    }
}

pub(crate) struct Rng {
    state: u32,
}

impl Rng {
    pub(crate) fn from_clock() -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos();
        // Mix bits slightly and guarantee non-zero state
        let seed = nanos.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    fn next_u32(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    /// Returns a u32 uniformly distributed in [min, max)
    pub(crate) fn next_u32_range(&mut self, min: u32, max: u32) -> u32 {
        debug_assert!(min < max, "min must be less than max");
        let range = max - min;

        // Lemire's fast debiased range reduction
        let mut x = self.next_u32();
        let mut m = (x as u64) * (range as u64);
        let mut l = m as u32;

        if l < range {
            let t = range.wrapping_neg() % range;
            while l < t {
                x = self.next_u32();
                m = (x as u64) * (range as u64);
                l = m as u32;
            }
        }

        min + (m >> 32) as u32
    }

    /// Returns a float in [0.0, 1.0)
    fn next_f32(&mut self) -> f32 {
        // Use the upper 24 bits for exact f32 mantissa precision
        (self.next_u32() >> 8) as f32 / (1u32 << 24) as f32
    }

    /// Returns a float in [min, max)
    pub(crate) fn next_f32_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }

    /// Accepts either `(f32, f32)` for uniform bounds across all axes,
    /// or `(Vec3, Vec3)` for per-axis bounds.
    pub(crate) fn next_vec3<B: Vec3Bounds>(&mut self, bounds: B) -> Vec3 {
        let (min, max) = bounds.into_vec3_bounds();
        Vec3::new(
            self.next_f32_range(min.x, max.x),
            self.next_f32_range(min.y, max.y),
            self.next_f32_range(min.z, max.z),
        )
    }
}
