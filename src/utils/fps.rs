use std::time::Instant;

/// Counts presented frames and reports the average frame rate about once per second.
pub(crate) struct FpsCounter {
    frames: u32,
    window_start: Instant,
}

impl FpsCounter {
    pub(crate) fn new() -> Self {
        Self {
            frames: 0,
            window_start: Instant::now(),
        }
    }

    /// Call once per presented frame. Returns `Some(fps)` when at least one second
    /// has passed since the last report, `None` otherwise.
    pub(crate) fn frame(&mut self) -> Option<f64> {
        self.frames += 1;
        let elapsed = self.window_start.elapsed().as_secs_f64();
        if elapsed < 1.0 {
            return None;
        }
        let fps = self.frames as f64 / elapsed;
        self.frames = 0;
        self.window_start = Instant::now();
        Some(fps)
    }
}
