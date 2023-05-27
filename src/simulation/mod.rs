#![feature(drain_filter)]

use std::{f32::consts::PI, num};

use sdl2::pixels::Color;
use sdl2::rect::Point;

use crate::asteroids;
use crate::vecmath::TransformationMatrix;
use crate::graphics::{self, renderGameOver, renderWonText};
use crate::{
    asteroids::Asteroid,
    collision, draw, hud,
    vecmath::{self, Vec2d},
};

struct Physics {
    gravity: f32, // force applied per second!
    gravity_direction: Vec2d,
}

pub struct Entity {
    position: Vec2d,
    rotation: f32,       // angle in rad
    direction: Vec2d,    // non normalized, has speed integrated!
    acceleration: Vec2d, // non normalized, has force integrated!
    update: bool,
}

pub struct Lander {
    entity_id: usize,
    //facing: Vec2d, // This is the direction the engine is facing, i.e. any thrust is opposite to this!
    facing: f32,
    drive_enabled: bool,
    rotation: f32,
}

#[derive(Clone, PartialEq)]
pub struct Missile 
{
    entity_id: usize,
    time_to_live: f32       // in seconds
}

#[derive(PartialEq)]
pub enum State {
    Running,
    Won,
    Lost,
}

pub struct World {
    p: Physics,
    entities: Vec<Entity>,
    missiles: Vec<Missile>,
    lander: Option<Lander>,
    asteroids: Vec<Asteroid>,
    hud: hud::Hud,
    game_state: State,
}

impl Missile {
    pub fn new(id: usize) -> Self {
        Self { entity_id: id, time_to_live: 5.0f32 }
    }
}

impl Entity {
    pub(crate) fn default() -> Self {
        Entity {
            position: Vec2d::default(),
            rotation: 0.0,
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
    pub fn get_transform(&self) -> TransformationMatrix {
        let pos = vecmath::TransformationMatrix::translation_v(self.position);
        let rot = vecmath::TransformationMatrix::rotate(self.rotation);
        return pos * rot;
    }
}

impl Physics {
    pub fn default() -> Self {
        Physics {
            gravity: 9.81 / 24.0,
            gravity_direction: Vec2d::new(0.0, 0.0),
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
            asteroids: Vec::new(),
            hud: hud::Hud::new(),
            game_state: State::Running,
            missiles: vec![]
        };

        w.create_asteroids();
        let landerId = w.create_entity();
        w.lander = Some(Lander {
            entity_id: landerId,
            facing: 0.0f32,
            drive_enabled: false,
            rotation: 0.0,
        });
        w
    }

    pub fn create_asteroids(&mut self) {
        for idx in 1..=3 {
            let id = self.create_entity();
            self.get_entity(id).set_position(Vec2d { x: (50 + 100 * idx) as f32, y: 50.0 });
            self.asteroids.push(Asteroid::new(id, idx));
        }
    }

    pub fn create_entity(&mut self) -> usize {
        let mut e = Entity::default();
        e.set_position(Vec2d::new(200.0, 300.0));
        self.entities.push(e);
        return self.entities.len() - 1;
    }

    pub fn create_missile(&mut self, pos: Vec2d) {
        let id = self.create_entity();
        self.get_entity(id).set_position(pos);
        self.missiles.push(Missile::new(id));
    }

    pub fn dismiss_dead_missiles(&mut self) {
        // self.missiles.retain(|&m| { m.time_to_live > 0.0 })
    }

    pub fn get_entity(&mut self, id: usize) -> &mut Entity {
        return &mut self.entities[id];
    }
    pub fn get_entity_immutable(&self, id: usize) -> &Entity {
        return &self.entities[id];
    }

    pub fn tick(&mut self, time_in_ms: f32, tick_resolution_in_ms: f32) {
        // Do physics (i.e. Gravity & Acceleration) tick
        self.p
            .tick(time_in_ms, tick_resolution_in_ms, &mut self.entities);

        let mut lander = self.lander.as_mut().unwrap();
        let mut disableThrust = false; 

        let mut next_angle = lander.facing + 45.0 * lander.rotation * (time_in_ms / 1000.0);
        lander.facing = next_angle; //Vec2d::from_angle(next_angle);

        self.thrust_toggle(disableThrust);


        // Do collision detection, fail if we collided with the environment
        // or a landingpad (in pad case: if velocity was too high)
        self.do_collision_detection();
    }

    fn get_lander_transform(&self, lander_pos: Vec2d, lander_rot: f32) -> TransformationMatrix
    {
        let scale = vecmath::TransformationMatrix::scale(graphics::LanderScale.x, graphics::LanderScale.y);
        let translate = vecmath::TransformationMatrix::translation_v(lander_pos);
        let rotation = vecmath::TransformationMatrix::rotate(lander_rot + PI / 2.0);
        let transform = translate * rotation * scale;
        transform
    }

    pub(crate) fn render(&mut self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        match self.game_state {
            State::Won => renderWonText(canvas),
            State::Lost => renderGameOver(canvas),
            State::Running => (),
        }

        for ast in self.asteroids.iter() {
            let draw_points = ast.get_transformed_hull(self.get_entity_immutable(ast.entity_id));
            draw::draw_lines(
                canvas,
                &draw_points,
                Color::RGB(255, 255, 255),
                true,
            )
            .unwrap();
        }
        self.renderHud(canvas);

        //draw the lander:
        let id;
        let thrust_enabled;
        let lander_rot;
        {
            // This scope makes sure, that we only keep the lander
            // borrowed as long as necessary
            let lander = self.lander.as_ref().unwrap();
            id = lander.entity_id;
            lander_rot = lander.facing.to_radians();
            thrust_enabled = lander.drive_enabled;
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
            geometry = transform.transform_many(&graphics::FlameA.to_vec());
            draw::draw_lines(canvas, &geometry, Color::RGB(255, 255, 255), true).unwrap();
        }

        for missile in self.missiles.iter() {
            let pos = self.get_entity_immutable(missile.entity_id).position;
            draw::draw_line(canvas, &pos, &(pos + Vec2d::new(1.0, 1.0)), Color::RGB(255, 255, 255)).unwrap();
        }
    }

    pub(crate) fn thrust_toggle(&mut self, enable: bool) {
        if self.game_state != State::Running {
            return;
        }
        let id;
        let thrust_dir;
        {
            // This scope makes sure, that we only keep the lander
            // borrowed as long as necessary
            let lander = self.lander.as_mut().unwrap();
            thrust_dir = Vec2d::from_angle(lander.facing.to_radians());       
            id = lander.entity_id;
            lander.drive_enabled = enable;
        }
        let entity = self.get_entity(id);
        if enable {
            entity.set_acceleration(thrust_dir * -5.0);
        } else {
            entity.set_acceleration(Vec2d::default());
        }
    }

    pub(crate) fn rotation_left_toggle(&mut self, enable: bool) {
        if self.game_state != State::Running {
            return;
        }
        let lander = self.lander.as_mut().unwrap();
        if enable == true {
            lander.rotation = -1.0;
        } else {
            lander.rotation = 0.0;
        }
    }

    pub(crate) fn rotation_right_toggle(&mut self, enable: bool) {
        if self.game_state != State::Running {
            return;
        }
        let lander = self.lander.as_mut().unwrap();
        if enable == true {
            lander.rotation = 1.0;
        } else {
            lander.rotation = 0.0;
        }
    }

    pub(crate) fn shoot(&mut self) {
        if let Some(lander) = self.lander.as_ref() {
            let id = lander.entity_id;
            let position = self.get_entity(id).position;
            self.create_missile(position);
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

            let transform = self.get_lander_transform(position, direction.angle());
            let bbox = transform.transform_many(&graphics::BBox.to_vec());


            let mut asteroids_to_delete = Vec::<usize>::new();
            let mut missiles_to_delete = Vec::<usize>::new();
            

            for a in self.asteroids.iter()
            {
                // Check collision against player:
                let e = self.get_entity_immutable(a.entity_id);
                let pts = &a.get_transformed_hull(e);
                let collision = collision::hit_test(position, pts);     // Primitive! This will only ever trigger, if the center of the starship is inside the asteroid.
                if collision
                {
                    self.game_state = State::Lost;
                }

                // Check collision against missiles
                for m in self.missiles.iter()
                {
                    let ent = self.get_entity_immutable(m.entity_id);
                    let projectile_collision = collision::hit_test(ent.position, pts);

                    // create two new asteroids, smaller than the previous one, but
                    // flying in other directions.
                    // also: Schedule this asteroid for deletion
                    if projectile_collision
                    {
                        asteroids_to_delete.push(a.entity_id);
                        missiles_to_delete.push(m.entity_id);
                    }

                }
            }

            // cleanup asteroids:
            let new_asteroids = self.asteroids.clone().into_iter().filter(|a| {!asteroids_to_delete.contains(&a.entity_id)});
            self.asteroids = new_asteroids.collect();
            let new_missiles = self.missiles.clone().into_iter().filter(|a| {!asteroids_to_delete.contains(&a.entity_id)});
            self.missiles = new_missiles.collect();

        }

    }

    fn renderHud(&mut self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        if let Some(lander) = self.lander.as_ref() {
            let id = lander.entity_id;
            let entity = self.get_entity(id);
            let position = entity.position;
            let direction = entity.direction;
            self.hud.update(position, direction, 0);
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
