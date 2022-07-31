use std::{f32::consts::PI, num};

use sdl2::pixels::Color;

use crate::graphics;
use crate::vecmath::TransformationMatrix;
use crate::{
    draw, hud,
    map::PointList,
    vecmath::{self, Vec2d},
    collision,
};

struct Physics {
    gravity: f32, // force applied per second!
    gravity_direction: Vec2d,
}

pub struct Entity {
    position: Vec2d,
    direction: Vec2d,    // non normalized, has speed integrated!
    acceleration: Vec2d, // non normalized, has force integrated!
    update: bool,
}

pub struct Lander {
    entity_id: usize,
    fuel: f32,     // in seconds!
    facing: Vec2d, // This is the direction the engine is facing, i.e. any thrust is opposite to this!
    drive_enabled: bool,
    rotation: f32,
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
            update: true,
        }
    }

    pub fn set_acceleration(&mut self, accel: Vec2d) {
        self.acceleration = accel;
    }

    pub fn set_position(&mut self, position: Vec2d) {
        self.position = position;
    }

    pub fn set_update(&mut self, update: bool) {
        self.update = update;
    }
}

impl Physics {
    pub fn default() -> Self {
        Physics {
            gravity: 9.81 / 24.0,
            gravity_direction: Vec2d::new(0.0, 1.0),
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
                    self.gravity_direction.clone() * (self.gravity * sim_time_in_seconds);

                if e.update {
                    e.direction = e.direction + gravity_fragment;
                    // update direction by appliying acceleration:
                    let accel_fragment = e.acceleration.clone() * (sim_time_in_seconds);
                    e.direction = e.direction + accel_fragment;
                    e.position = e.position + e.direction.clone() * (sim_time_in_seconds);
                }

                // TBD: Check if something like a terminal velocity would be a good idea
                // -> This would probably make the game a bit easier and also make the physics
                // simulation more robust
            }
        }
    }
}

impl World {
    pub fn new(window_width: u32, window_height: u32) -> Self {
        let mut w = World {
            p: Physics::default(),
            entities: Vec::new(),
            lander: None,
            map: PointList::new(window_width as f32, (window_height as f32) / 3.0),
            hud: hud::Hud::new(),
        };
        w.map.set_window_height(window_height as f32);
        let landerId = w.create_entity();
        w.lander = Some(Lander {
            entity_id: landerId,
            fuel: 20.0,
            facing: Vec2d::new(0.0, 1.0),
            drive_enabled: false,
            rotation: 0.0,
        });
        w
    }

    pub fn create_entity(&mut self) -> usize {
        let mut e = Entity::default();
        e.set_position(Vec2d::new(200.0, 300.0));
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
        let mut disableThrust = false; 
        if lander.drive_enabled {
            lander.fuel -= time_in_ms / 1000.0;
            if lander.fuel <= 0.0
            {
                lander.drive_enabled = false;
                disableThrust = true;
            }
        }

        let mut next_angle = lander.facing.angle() + lander.rotation * (time_in_ms / 1000.0);
        lander.facing = Vec2d::from_angle(next_angle);

        self.thrust_toggle(disableThrust);


        // Do collision detection, fail if we collided with the environment
        // or a landingpad (in pad case: if velocity was too high)
        self.do_collision_detection();
    }

    fn get_lander_transform(&self, lander_pos: Vec2d, lander_rot: Vec2d) -> TransformationMatrix
    {
        let scale = vecmath::TransformationMatrix::scale(graphics::LanderScale.x, graphics::LanderScale.y);
        let translate = vecmath::TransformationMatrix::translation_v(lander_pos);
        let rotation = vecmath::TransformationMatrix::rotate(lander_rot.angle() + PI / 2.0);
        let transform = translate * rotation * scale;
        transform
    }

    pub(crate) fn render(&mut self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {

        draw::draw_lines(
            canvas,
            &self.map.get_values(),
            Color::RGB(255, 255, 255),
            false,
        )
        .unwrap();
        self.renderHud(canvas);

        //draw the lander:
        let id;
        let thrust_enabled;
        let lander_rot;
        let fuel;
        {
            // This scope makes sure, that we only keep the lander
            // borrowed as long as necessary
            let lander = self.lander.as_ref().unwrap();
            id = lander.entity_id;
            lander_rot = lander.facing;
            thrust_enabled = lander.drive_enabled;
            fuel = lander.fuel;
        }
        let entity = self.get_entity(id);
        let lander_pos = entity.position;

        let transform = self.get_lander_transform(lander_pos, lander_rot);
        let items = [
            &graphics::LanderTop,
            &graphics::LanderMiddle,
            &graphics::LanderBottom,
            &graphics::LanderDrive,
            &graphics::BBox
        ];
        for lander_part in items.iter() {
            let geometry = transform.transform_many(&lander_part.to_vec());
            draw::draw_lines(canvas, &geometry, Color::RGB(255, 255, 255), true).unwrap();
        }

        if thrust_enabled {
            let geometry;
            if ((fuel * 10.0) as i32) % 2 == 0 {
                geometry = transform.transform_many(&graphics::FlameA.to_vec());
            } else {
                geometry = transform.transform_many(&graphics::FlameB.to_vec());
            }
            draw::draw_lines(canvas, &geometry, Color::RGB(255, 255, 255), true).unwrap();
        }
    }

    pub(crate) fn thrust_toggle(&mut self, enable: bool) {
        let id;
        let thrust_dir;
        let hasFuel;
        {
            // This scope makes sure, that we only keep the lander
            // borrowed as long as necessary
            let lander = self.lander.as_mut().unwrap();
            thrust_dir = lander.facing;       
            id = lander.entity_id;
            hasFuel = lander.fuel > 0.0;
            lander.drive_enabled = hasFuel && enable;
        }
        let entity = self.get_entity(id);
        if enable && hasFuel{
            entity.set_acceleration(thrust_dir * -5.0);
        } else {
            entity.set_acceleration(Vec2d::default());
        }
    }

    pub(crate) fn rotation_left_toggle(&mut self, enable: bool) {
        let lander = self.lander.as_mut().unwrap();
        if enable == true {
            lander.rotation = -1.0;
        } else {
            lander.rotation = 0.0;
        }
    }

    pub(crate) fn rotation_right_toggle(&mut self, enable: bool) {
        let lander = self.lander.as_mut().unwrap();
        if enable == true {
            lander.rotation = 1.0;
        } else {
            lander.rotation = 0.0;
        }
    }

    fn do_collision_detection(&mut self)
    {
        if let Some(lander) = self.lander.as_ref() {
            let id = lander.entity_id;
            let position;
            let direction;
            {
                let entity = self.get_entity(id);
                position = entity.position;
                direction = entity.direction;
            }

            let transform = self.get_lander_transform(position, direction);
            let bbox = transform.transform_many(&graphics::BBox.to_vec());

            if let Some(collision) = collision::detect_collision(
                bbox, self.map.get_values()) {
                    let entity = self.get_entity(id);
                    entity.set_update(false);
            }
        }
    }

    fn renderHud(&mut self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        if let Some(lander) = self.lander.as_ref() {
            let fuel = lander.fuel;
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
