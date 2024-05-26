use crate::vecmath::Vec2d;

use super::{BorderBehavior, Entity, WORLD_SIZE};

pub struct Physics {
    gravity: f32, // force applied per second!
    gravity_direction: Vec2d,
}

impl Physics {
    pub fn default() -> Self {
        Physics {
            gravity: 9.81 / 24.0,
            gravity_direction: Vec2d::new(0.0, 0.0),
        }
    }

    pub fn physics_tick(
        &self,
        time_in_ms: f32,
        tick_resolution_in_ms: f32,
        entities: &mut Vec<Entity>,
    ) {
        let mut num_ticks = (time_in_ms / tick_resolution_in_ms) as u32;
        if num_ticks == 0 {
            num_ticks = 1;
        }

        //println!("Ticks {}", num_ticks);

        // Apply gravity and acceleration to each entity,
        // Apply resulting speed to position of entity
        let tick_width = (time_in_ms / num_ticks as f32) / 1000.0f32;
        for _ in 0..num_ticks {
            for e in entities.iter_mut() {
                let sim_time_in_seconds = tick_width;

                // update direction by applying gravity:
                let gravity_fragment =
                    self.gravity_direction.clone() * (self.gravity * sim_time_in_seconds);

                if e.update {
                    e.direction = e.direction + gravity_fragment;
                    // update direction by applying acceleration:
                    let accel_fragment = e.acceleration.clone() * (sim_time_in_seconds);
                    e.direction = e.direction + accel_fragment;

                    e.direction = if e.direction.len() > e.max_velocity {
                        e.direction.normalized() * e.max_velocity
                    } else {
                        e.direction
                    };
                    let mut new_pos = e.position + e.direction.clone() * (sim_time_in_seconds);

                    if new_pos.x < 0.0
                        || new_pos.y < 0.0
                        || new_pos.x > WORLD_SIZE.x
                        || new_pos.y > WORLD_SIZE.y
                    {
                        match e.border_behavior {
                            BorderBehavior::Dismiss => {
                                // TODO: destroy missile/entity
                            }
                            BorderBehavior::Bounce => {
                                e.direction = e.direction * -1.0;
                                new_pos = e.position + e.direction.clone() * (sim_time_in_seconds);
                            }
                            BorderBehavior::BounceSlowdown => {
                                e.direction = e.direction * -0.2;
                                new_pos = e.position + e.direction.clone() * (sim_time_in_seconds);
                            }
                        }
                    }

                    e.position = new_pos;
                }

                // TBD: Check if something like a terminal velocity would be a good idea
                // -> This would probably make the game a bit easier and also make the physics
                // simulation more robust
            }
        }
    }
}
