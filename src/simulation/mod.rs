use std::num;

use crate::vecmath::Vec2d;

struct Physics {
    gravity: f32, // force applied per second!
    gravity_direction: Vec2d,
}

struct Entity {
    position: Vec2d,
    direction: Vec2d,    // non normalized, has speed integrated!
    acceleration: Vec2d, // non normalized, has force integrated!
}

impl Entity {
    pub(crate) fn default() -> Self {
        Entity { 
            position: Vec2d::default(), 
            direction: Vec2d::default(), 
            acceleration: Vec2d::default() 
        }
    }
}

impl Physics {
    pub fn default() -> Self {
        Physics {
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
        for _ in 0..num_ticks {
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


mod tests {
    use crate::{simulation, vecmath::Vec2d};

    use super::{Physics, Entity};

    #[test]
    fn can_apply_gravity()
    {
        let w = Physics{gravity: 1.0, gravity_direction: Vec2d::new(0.0, -1.0)};

        let mut e = Entity::default();

        let mut v = vec![e];

        w.tick(1000.0, 1000.0, &mut v);
        assert_eq!(v[0].position.y, -1.0);

    }

    #[test]
    fn can_apply_acceleration()
    {
        let w = Physics{gravity: 1.0, gravity_direction: Vec2d::default()};

        let mut e = Entity::default();
        e.acceleration = Vec2d::new(1.0, 0.0);
        let mut v = vec![e];
        

        w.tick(1000.0, 1000.0, &mut v);
        assert_eq!(v[0].position.x, 1.0);
        assert_eq!(v[0].direction.x, 1.0);

    }
}

