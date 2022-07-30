use std::num;

use sdl2::pixels::Color;

use crate::{draw, map::PointList, vecmath::{Vec2d, self}, hud};
use crate::graphics;

struct Physics {
    gravity: f32, // force applied per second!
    gravity_direction: Vec2d,
}

pub struct Entity {
    position: Vec2d,
    direction: Vec2d,    // non normalized, has speed integrated!
    acceleration: Vec2d, // non normalized, has force integrated!
}

pub struct Lander {
    entity_id: usize,
    fuel: f32,     // in seconds!
    facing: Vec2d, // This is the direction the engine is facing, i.e. any thrust is opposite to this!
}

pub struct World {
    p: Physics,
    entities: Vec<Entity>,
    lander: Option<Lander>,
    map: PointList,
    hud: hud::Hud,
}

impl Entity {
    pub(crate) fn default() -> Self {
        Entity {
            position: Vec2d::default(),
            direction: Vec2d::default(),
            acceleration: Vec2d::default(),
        }
    }

    pub fn set_acceleration(&mut self, accel: Vec2d) {
        self.acceleration = accel;
    }

    pub fn set_position(&mut self, position: Vec2d) {
        self.position = position;
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

                // TBD: Check if something like a terminal velocity would be a good idea
                // -> This would probably make the game a bit easier and also make the physics
                // simulation more robust
            }
        }
    }
}

impl World {
    pub fn new() -> Self {
        let mut w = World {
            p: Physics::default(),
            entities: Vec::new(),
            lander: None,
            map: PointList::new(800.0, 500.0),
            hud: hud::Hud::new(),
        };
        let landerId = w.create_entity();
        w.lander = Some(Lander {
            entity_id: landerId,
            fuel: 20.0,
            facing: Vec2d::new(0.0, -1.0),
        });
        w
    }

    pub fn create_entity(&mut self) -> usize {
        let mut e = Entity::default();
        self.entities.push(e);
        return self.entities.len() - 1;
    }

    pub fn get_entity(&mut self, id: usize) -> &mut Entity {
        return &mut self.entities[id];
    }

    pub fn tick(&mut self, time_in_ms: f32, tick_resolution_in_ms: f32) {
        // Do physics (i.e. Gravity & Acceleration) tick
        self.p
            .tick(time_in_ms, tick_resolution_in_ms, &mut self.entities);

        // Consume fuel
        let mut lander = self.lander.as_mut().unwrap();
        lander.fuel -= time_in_ms / 1000.0;

        // Do collision detection, fail if we collided with the environment
        // or a landingpad (in pad case: if velocity was too high)
        self.do_collision_detection();
    }

    pub(crate) fn render(&mut self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        draw::draw_lines(canvas, &self.map.get_values(), Color::RGB(255, 255, 255), false).unwrap();
        self.renderHud(canvas);

        //draw the lander:
        let id;
        {
            // This scope makes sure, that we only keep the lander
            // borrowed as long as necessary
            let lander = self.lander.as_ref().unwrap();
            id = lander.entity_id;
        }
        let entity = self.get_entity(id);
        let lander_pos = entity.position;

        let scale = vecmath::TransformationMatrix::scale(10.0,10.0);
        let translate =vecmath::TransformationMatrix::translation(100.0, 200.0);
        //let translate =vecmath::TransformationMatrix::unit();
        let transform = translate * scale;
        let items = [&graphics::LanderTop, &graphics::LanderMiddle, &graphics::LanderBottom, &graphics::LanderDrive];
        for lander_part in items.iter()
        {
            let geometry = transform.transform_many(&lander_part.to_vec());
            draw::draw_vec_strip(canvas, &geometry, Color::RGB(255, 255, 255), true);
        }

    }

    pub(crate) fn thrust_toggle(&mut self, enable: bool) {
        let id;
        let thrust_dir;
        {
            // This scope makes sure, that we only keep the lander
            // borrowed as long as necessary
            let lander = self.lander.as_ref().unwrap();
            thrust_dir = lander.facing;
            id = lander.entity_id;
        }
        let entity = self.get_entity(id);
        if enable {
            entity.set_acceleration(thrust_dir * -1.0);
        } else {
            entity.set_acceleration(Vec2d::default());
        }
    }

    pub(crate) fn rotation_left_toggle(&self, enable: bool) {}

    pub(crate) fn rotation_right_toggle(&self, enable: bool) {}

    fn do_collision_detection(&self) {}

    fn renderHud(&mut self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>)
    {
        if let Some(lander) = self.lander.as_ref() {
            let fuel  = lander.fuel;
            let id = lander.entity_id;
            let entity = self.get_entity(id);
            let position = entity.position;
            let direction = entity.direction;
            self.hud.update(position, direction, fuel, 0);
        }
        self.hud.render(canvas);
    }
}

mod tests {
    use crate::{simulation, vecmath::Vec2d};

    use super::{Entity, Physics};

    #[test]
    fn can_apply_gravity() {
        let w = Physics {
            gravity: 1.0,
            gravity_direction: Vec2d::new(0.0, -1.0),
        };

        let mut e = Entity::default();

        let mut v = vec![e];

        w.tick(1000.0, 1000.0, &mut v);
        assert_eq!(v[0].position.y, -1.0);
    }

    #[test]
    fn can_apply_acceleration() {
        let w = Physics {
            gravity: 1.0,
            gravity_direction: Vec2d::default(),
        };

        let mut e = Entity::default();
        e.acceleration = Vec2d::new(1.0, 0.0);
        let mut v = vec![e];

        w.tick(1000.0, 1000.0, &mut v);
        assert_eq!(v[0].position.x, 1.0);
        assert_eq!(v[0].direction.x, 1.0);
    }
}
