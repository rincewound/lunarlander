use std::num;

use crate::vecmath::Vec2d;

struct World {
    gravity: f32, // force applied per second!
    gravity_direction: Vec2d,
}

struct Entity {
    position: Vec2d,
    direction: Vec2d,    // non normalized, has speed integrated!
    acceleration: Vec2d, // non normalized, has force integrated!
}

impl World {
    pub fn default() -> Self {
        World {
            gravity: 9.81,
            gravity_direction: Vec2d::new(0.0, -1.0),
        }
    }

    pub fn tick(&self, time_in_ms: f32, tick_resolution_in_ms: f32, entities: &mut Vec<Entity>) {
        let mut num_ticks = (time_in_ms / tick_resolution_in_ms) as u32;
        if num_ticks == 0 {
            num_ticks = 1;
        }

        // Apply gravity and acceleration to each entity,
        // Apply resulting speed to position of entity
        for i in 0..num_ticks {
            for e in entities.iter_mut() {
                let sim_time_in_seconds = time_in_ms / 1000.0;

                // update direction by applying gravity:
                let gravity_fragment =
                    self.gravity_direction.clone() * (self.gravity / sim_time_in_seconds);
                e.direction = e.direction + gravity_fragment;
                // update direction by appliying acceleration:
                let accel_fragment = e.acceleration.clone() * (sim_time_in_seconds);
                e.direction = e.direction + accel_fragment;
                e.position = e.position + e.direction.clone() * (sim_time_in_seconds);
            }
        }
    }
}
