use glam::Vec3;

/// Converts an sRGB colour given in 0–255 per channel to linear 0–1 values,
/// using the exact piecewise sRGB transfer curve.
pub(super) fn srgb_to_linear(color: Vec3) -> Vec3 {
    fn channel(c: f32) -> f32 {
        let s = c / 255.0;
        if s <= 0.04045 {
            s / 12.92
        } else {
            ((s + 0.055) / 1.055).powf(2.4)
        }
    }

    Vec3::new(channel(color.x), channel(color.y), channel(color.z))
}
