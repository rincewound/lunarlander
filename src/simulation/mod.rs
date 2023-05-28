#![feature(drain_filter)]

use std::collections::HashMap;
use std::{f32::consts::PI, num};

use rand::Rng;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::{Texture, BlendMode};

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
    id: usize,
    position: Vec2d,
    angle: f32,       // angle in rad
    direction: Vec2d,    // non normalized, has speed integrated!
    acceleration: Vec2d, // non normalized, has force integrated!
    update: bool,
}

pub struct Lander {
    entity_id: usize,
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

pub struct Star{
    pos: Vec2d,
    layer: u8
}

pub struct World {
    p: Physics,
    screen_space_transform: TransformationMatrix,
    next_entity_id: usize,
    entities: Vec<Entity>,
    missiles: Vec<Missile>,
    starfield: Vec<Star>,
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
    pub(crate) fn default(id: usize) -> Self {
        Entity {
            id,
            position: Vec2d::default(),
            angle: 0.0,
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
        let rot = vecmath::TransformationMatrix::rotate(self.angle);
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
            screen_space_transform: TransformationMatrix::unit(),
            next_entity_id: 0,
            p: Physics::default(),
            entities: Vec::new(),
            lander: None,
            asteroids: Vec::new(),
            hud: hud::Hud::new(),
            game_state: State::Running,
            missiles: vec![],
            starfield: Self::make_starfield()

        };

        w.create_asteroids();
        let landerId = w.create_entity();
        w.lander = Some(Lander {
            entity_id: landerId,
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
        let entity_id = self.next_entity_id;
        self.next_entity_id += 1;
        let mut e = Entity::default(entity_id);
        e.set_position(Vec2d::new(200.0, 300.0));
        self.entities.push(e);
        return entity_id;
    }

    pub fn create_missile(&mut self, pos: Vec2d, direction: Vec2d) {
        let id = self.create_entity();
        let entity = self.get_entity(id);
        entity.set_position(pos);
        entity.direction = direction * -40.0;
        self.missiles.push(Missile::new(id));
    }

    fn garbage_collect_entities(&mut self,  ids_to_remove: &Vec<usize>)
    {
        self.entities.retain(|x| !ids_to_remove.contains(&x.id));
    }

    pub fn dismiss_dead_missiles(&mut self) {
        let ids_to_remove: Vec<usize> = self.missiles.iter()
                                        .filter(|x| {x.time_to_live <= 0.0})
                                        .map(|m| m.entity_id).collect();
        self.garbage_collect_entities(&ids_to_remove);
        self.missiles.retain(|m| { m.time_to_live > 0.0 });
    }

    fn entity_id_to_index(&self, id: usize) -> usize
    {
        let mut idx = 0;
        for e in self.entities.iter()
        {
            if e.id == id
            {
                return idx;
            }
            idx +=1;
        }
        panic!("Entity with id {} does not exist", id)
    }

    pub fn get_entity(&mut self, id: usize) -> &mut Entity {
        let entity_index = self.entity_id_to_index(id);
        return &mut self.entities[entity_index];
    }
    pub fn get_entity_immutable(&self, id: usize) -> &Entity {
        let entity_index = self.entity_id_to_index(id);
        return &self.entities[entity_index];
    }

    pub fn tick(&mut self, time_in_ms: f32, tick_resolution_in_ms: f32) {
        // Do physics (i.e. Gravity & Acceleration) tick
        self.p
            .tick(time_in_ms, tick_resolution_in_ms, &mut self.entities);

        let lander = self.lander.as_ref().unwrap();
        let entity = self.get_entity_immutable(lander.entity_id);
        let next_angle = entity.angle + (45.0 * lander.rotation * (time_in_ms / 1000.0)).to_radians();
        let mut entity = self.get_entity(lander.entity_id);
        entity.angle = next_angle; //Vec2d::from_angle(next_angle);

        let disable_thrust = false;     
        self.thrust_toggle(disable_thrust);
        self.missile_tick(time_in_ms);
        self.dismiss_dead_missiles();

        // Do collision detection, fail if we collided with the environment
        // or a landingpad (in pad case: if velocity was too high)
        self.do_collision_detection();
    }

    pub(crate) fn render(&mut self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, textures: &HashMap<String, Texture>) {
        match self.game_state {
            State::Won => renderWonText(canvas),
            State::Lost => renderGameOver(canvas),
            State::Running => (),
        }

        self.render_starfield(canvas, textures);

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
        {
            // This scope makes sure, that we only keep the lander
            // borrowed as long as necessary
            let lander = self.lander.as_ref().unwrap();
            id = lander.entity_id;
            thrust_enabled = lander.drive_enabled;
        }
        let entity = self.get_entity_immutable(id);

        let scale = vecmath::TransformationMatrix::scale(graphics::LanderScale.x, graphics::LanderScale.y);
        let entity_trans = entity.get_transform();
        // fix orientation of lander and rotate 90 deg
        let offset = vecmath::TransformationMatrix::rotate(PI / 2.0);
        let transform = entity_trans * scale * offset;
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
            let angle;
            {
                let lander = self.lander.as_ref().unwrap();
                let entity = self.get_entity_immutable(lander.entity_id);
                angle = entity.angle;
            }
            let lander = self.lander.as_mut().unwrap();
            thrust_dir = Vec2d::from_angle(angle);
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
            let entity = self.get_entity_immutable(id);
            let position = entity.position;
            let direction = Vec2d::from_angle(entity.angle);
            self.create_missile(position, direction);
        }
    }

    fn missile_tick(&mut self, time_in_ms: f32) {
        for missile in self.missiles.iter_mut() {
            missile.time_to_live -= time_in_ms / 1000.0f32;
        }
    }

    fn do_collision_detection(&mut self)
    {
        if let Some(lander) = self.lander.as_ref() {
            let id = lander.entity_id;
            let entity = self.get_entity_immutable(id);
            let position;
            let direction;
            {
                position = entity.position;
                direction = entity.direction;
            }

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
            self.garbage_collect_entities(&asteroids_to_delete);
            let new_missiles = self.missiles.clone().into_iter().filter(|a| {!missiles_to_delete.contains(&a.entity_id)});
            self.garbage_collect_entities(&missiles_to_delete);
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

    fn render_starfield(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, textures: &HashMap<String, Texture>) 
    {
        let lander = self.lander.as_ref().unwrap();
        let lander_pos = self.get_entity_immutable(lander.entity_id).position.clone();

        let texture = textures.get("star").unwrap();
        for star in self.starfield.iter()
        {
            let starpos = star.pos.clone() - lander_pos * (0.75 + star.layer as f32) as f32;
            let _ = canvas.copy(texture, None, sdl2::rect::Rect::new(starpos.x as i32, starpos.y as i32, 16 / (1 + star.layer as u32), 16/ (1 + star.layer as u32)));
        }
    }

    fn make_starfield() -> Vec<Star> {
        let mut output = Vec::<Star>::new();
        let mut rnd = rand::thread_rng();
        for _ in 0..2000
        {
            let s = Star {
                pos: Vec2d::new(rnd.gen_range(-1800..1800) as f32, rnd.gen_range(-1600..1600) as f32),
                layer:rnd.gen_range(0..3) as u8   
            };
            output.push(s);

        }
        output
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
