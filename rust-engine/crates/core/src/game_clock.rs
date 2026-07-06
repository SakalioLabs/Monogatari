//! Game clock for tracking time and frame rate.

use std::time::Instant;

/// Tracks game time, delta time, and frame rate.
pub struct GameClock {
    #[allow(dead_code)]
    start_time: Instant,
    last_frame: Instant,
    delta_time: f32,
    total_time: f32,
    frame_count: u64,
    fps: f32,
    fps_update_timer: f32,
    fps_frame_count: u32,
    fixed_delta_time: f32,
    fixed_update_accumulator: f32,
}

impl GameClock {
    /// Create a new game clock with the given fixed update rate.
    pub fn new(fixed_update_rate: f32) -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            last_frame: now,
            delta_time: 0.0,
            total_time: 0.0,
            frame_count: 0,
            fps: 0.0,
            fps_update_timer: 0.0,
            fps_frame_count: 0,
            fixed_delta_time: 1.0 / fixed_update_rate,
            fixed_update_accumulator: 0.0,
        }
    }

    /// Update the clock. Call once per frame.
    pub fn tick(&mut self) {
        let now = Instant::now();
        self.delta_time = (now - self.last_frame).as_secs_f32();
        self.last_frame = now;
        self.total_time += self.delta_time;
        self.frame_count += 1;

        // Update FPS counter
        self.fps_update_timer += self.delta_time;
        self.fps_frame_count += 1;
        if self.fps_update_timer >= 1.0 {
            self.fps = self.fps_frame_count as f32 / self.fps_update_timer;
            self.fps_update_timer = 0.0;
            self.fps_frame_count = 0;
        }

        // Accumulate for fixed update
        self.fixed_update_accumulator += self.delta_time;
    }

    /// Returns true if a fixed update step should run.
    pub fn should_fixed_update(&mut self) -> bool {
        if self.fixed_update_accumulator >= self.fixed_delta_time {
            self.fixed_update_accumulator -= self.fixed_delta_time;
            true
        } else {
            false
        }
    }

    /// Time elapsed since last frame in seconds.
    pub fn delta_time(&self) -> f32 {
        self.delta_time
    }

    /// Total elapsed game time in seconds.
    pub fn total_time(&self) -> f32 {
        self.total_time
    }

    /// Total number of frames rendered.
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Current frames per second.
    pub fn fps(&self) -> f32 {
        self.fps
    }

    /// Fixed delta time for physics/logic updates.
    pub fn fixed_delta_time(&self) -> f32 {
        self.fixed_delta_time
    }
}

impl Default for GameClock {
    fn default() -> Self {
        Self::new(60.0)
    }
}
