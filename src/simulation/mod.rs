#![feature(drain_filter)]

use std::collections::HashMap;
use std::{f32::consts::PI, num};

use rand::Rng;
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Texture};

use crate::asteroids::{self, MAX_SCALE};
use crate::draw::draw_lines;
use crate::graphics::{self, render_game_over, render_won_text};
use crate::sound;
use crate::vecmath::TransformationMatrix;
use crate::{
    asteroids::Asteroid,
    collision, draw, hud,
    vecmath::{self, Vec2d},
};

const VELOCITY_SPACESHIP: f32 = 50.0;
const VELOCITY_ASTEROID: f32 = 30.0;
const VELOCITY_MISSILE: f32 = 90.0;

struct Physics {
    gravity: f32, // force applied per second!
    gravity_direction: Vec2d,
}

pub enum BorderBehavior {
    Dismiss,
    Bounce,
    BounceSlowdown,
}

pub enum DirectionKey {
    Up,
    Down,
    Left,
    Right,
}

pub struct Entity {
    id: usize,
    position: Vec2d,
    angle: f32,          // angle in rad
    direction: Vec2d,    // non normalized, has speed integrated!
    acceleration: Vec2d, // non normalized, has force integrated!
    max_velocity: f32,
    border_behavior: BorderBehavior,
    update: bool,
}

pub struct Lander {
    entity_id: usize,
    drive_enabled: bool,
}

#[derive(Clone, PartialEq)]
pub struct Missile {
    entity_id: usize,
    time_to_live: f32, // in seconds
}

#[derive(PartialEq)]
pub enum State {
    Running,
    Won,
    Lost,
}

pub struct Star {
    pos: Vec2d,
    layer: u8,
}

pub struct World {
    p: Physics,
    next_entity_id: usize,
    entities: Vec<Entity>,
    missiles: Vec<Missile>,
    starfield: Vec<Star>,
    lander: Lander,
    asteroids: Vec<Asteroid>,
    hud: hud::Hud,
    game_state: State,
    score: u32,

    screen_shake_frames: usize,
    screen_shake_strength: f32,
    screen_size: Vec2d,
    sound: sound::Sound,
    whiteout_frames: u32, // number of frames to draw white, when large asteroids are destroyed
}

impl Missile {
    pub fn new(id: usize) -> Self {
        Self {
            entity_id: id,
            time_to_live: 5.0f32,
        }
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
            max_velocity: 0.0,
            border_behavior: BorderBehavior::Dismiss,
            update: true,
        }
    }

    pub fn set_acceleration(&mut self, accel: Vec2d) {
        self.acceleration = accel;
    }

    pub fn set_position(&mut self, position: Vec2d) {
        self.position = position;
    }

    fn set_direction(&mut self, direction: Vec2d) {
        self.direction = direction;
    }

    pub fn set_update(&mut self, update: bool) {
        self.update = update;
    }
    pub fn get_transform(&self) -> TransformationMatrix {
        let pos = vecmath::TransformationMatrix::translation_v(self.position);
        let rot = vecmath::TransformationMatrix::rotate(self.angle);
        return pos * rot;
    }

    pub fn get_screenspace_transform(
        &self,
        screenspace_transform: TransformationMatrix,
    ) -> TransformationMatrix {
        let pos = vecmath::TransformationMatrix::translation_v(self.position);
        let rot = vecmath::TransformationMatrix::rotate(self.angle);
        return pos * screenspace_transform * rot;
    }

    pub fn get_id(&self) -> usize {
        return self.id;
    }

    fn velocity(&self) -> f32 {
        return self.direction.len();
    }
}

const WorldSize: Vec2d = Vec2d {
    x: 4.0 * 800.0,
    y: 4.0 * 600.0,
};

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
                        || new_pos.x > WorldSize.x
                        || new_pos.y > WorldSize.y
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

impl World {
    pub fn new(window_width: u32, window_height: u32) -> Self {
        let lander = Lander {
            entity_id: 0,
            drive_enabled: false,
        };

        let mut lander_entity = Entity::default(0);
        lander_entity.set_position(WorldSize / 2.0);
        lander_entity.max_velocity = VELOCITY_SPACESHIP;
        lander_entity.border_behavior = BorderBehavior::BounceSlowdown;

        let w = World {
            next_entity_id: 1,
            p: Physics::default(),
            entities: vec![lander_entity],
            lander,
            asteroids: Vec::new(),
            hud: hud::Hud::new(),
            game_state: State::Running,
            missiles: vec![],
            starfield: Self::make_starfield(),
            score: 0,
            sound: sound::Sound::new(),
            screen_shake_frames: 0,
            screen_shake_strength: 0.0,
            screen_size: Vec2d {
                x: window_width as f32,
                y: window_height as f32,
            },
            whiteout_frames: 0,
        };

        w
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
        entity.position = pos;
        entity.direction = direction;
        entity.max_velocity = VELOCITY_MISSILE;
        entity.border_behavior = BorderBehavior::Dismiss;
        self.missiles.push(Missile::new(id));
    }

    fn garbage_collect_entities(&mut self, ids_to_remove: &Vec<usize>) {
        self.entities.retain(|x| !ids_to_remove.contains(&x.id));
    }

    pub fn dismiss_dead_missiles(&mut self) {
        let ids_to_remove: Vec<usize> = self
            .missiles
            .iter()
            .filter(|x| x.time_to_live <= 0.0)
            .map(|m| m.entity_id)
            .collect();
        self.garbage_collect_entities(&ids_to_remove);
        self.missiles.retain(|m| m.time_to_live > 0.0);
    }

    fn entity_id_to_index(&self, id: usize) -> usize {
        let mut idx = 0;
        for e in self.entities.iter() {
            if e.id == id {
                return idx;
            }
            idx += 1;
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

        //TODO: maybe use this to update lander angle smoothly
        //let rotation = self.lander.rotation;
        //let mut entity = self.get_entity(self.lander.entity_id);
        //entity.angle = entity.angle + (180.0 * rotation * (time_in_ms / 1000.0)).to_radians();
        // if the drive is still enabled and we changed the angle we must update the thrust
        //self.thrust_toggle(self.lander.drive_enabled);

        self.missile_tick(time_in_ms);
        self.dismiss_dead_missiles();

        // Do collision detection, fail if we collided with the environment
        // or a landingpad (in pad case: if velocity was too high)
        self.do_collision_detection();
        self.sound.play_background_music();
    }

    pub(crate) fn render(
        &mut self,
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
        textures: &HashMap<String, Texture>,
    ) {
        match self.game_state {
            State::Won => render_won_text(canvas, self.screen_size / 2.0),
            State::Lost => render_game_over(canvas, self.screen_size / 2.0),
            State::Running => (),
        }

        let mut ShakeTransForm = TransformationMatrix::unit();
        if self.screen_shake_frames > 0 {
            self.screen_shake_frames -= 1;
            let mut rnd = rand::thread_rng();
            ShakeTransForm = ShakeTransForm
                * TransformationMatrix::translation(
                    rnd.gen_range(0..self.screen_shake_strength as i32) as f32 / 2.0,
                    rnd.gen_range(0..self.screen_shake_strength as i32) as f32 / 2.0,
                )
        }

        let lander_entity = self.get_entity_immutable(self.lander.entity_id);

        let mut screen_space_transform = TransformationMatrix::unit();
        screen_space_transform = screen_space_transform
            * TransformationMatrix::translation_v(lander_entity.position * -1.0)
            * ShakeTransForm
            * TransformationMatrix::translation_v(self.screen_size / 2.0); // center to screen

        //self.render_starfield(canvas, textures);
        //self.render_asteroids(screen_space_transform, canvas);
        self.render_starship(lander_entity, screen_space_transform, canvas, textures);
        self.render_missiles(screen_space_transform, canvas);

        if self.whiteout_frames > 0 {
            self.whiteout_frames -= 1;
            let alpha = 255 * (1.0 - (10.0 / self.whiteout_frames as f32)) as u8;
            canvas.set_draw_color(Color::RGBA(255, 255, 255, alpha));
            canvas.set_blend_mode(BlendMode::Mul);
            canvas.fill_rect(Rect::new(0, 0, 800, 600));
            canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));
            canvas.set_blend_mode(BlendMode::None);
        }

        self.renderHud(canvas);
    }

    fn render_missiles(
        &mut self,
        screen_space_transform: TransformationMatrix,
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    ) {
        for missile in self.missiles.iter() {
            let pos = screen_space_transform
                .transform(&self.get_entity_immutable(missile.entity_id).position);
            let _ = draw::draw_line(
                canvas,
                &pos,
                &(pos + Vec2d::new(1.0, 1.0)),
                Color::RGB(255, 255, 255),
            );
        }
    }

    pub(crate) fn update_window_size(&mut self, width: f32, height: f32) {
        self.screen_size.x = width;
        self.screen_size.y = height;
    }

    pub(crate) fn thrust_toggle(&mut self, enable: bool) {
        if self.game_state != State::Running {
            return;
        }
        self.lander.drive_enabled = enable;
        let entity = self.get_entity(self.lander.entity_id);
        if enable {
            let thrust_dir = Vec2d::from_angle(entity.angle);
            entity.set_acceleration(thrust_dir * -5.0);
            // self.sound.accelerate();
            self.screen_shake_frames = 10;
            self.screen_shake_strength = 4.0;
        } else {
            entity.set_acceleration(Vec2d::default());
        }
    }

    pub(crate) fn direction_toggle(&mut self, dir: DirectionKey, enable: bool) {
        if self.game_state != State::Running {
            return;
        }
        let mut entity = self.get_entity(self.lander.entity_id);
        let dir_vec = match dir {
            DirectionKey::Up => Vec2d::new(0.0, -1.0),
            DirectionKey::Down => Vec2d::new(0.0, 1.0),
            DirectionKey::Left => Vec2d::new(-1.0, 0.0),
            DirectionKey::Right => Vec2d::new(1.0, 0.0),
        };
        // set direction based on toggled keys
        let new_dir = if enable {
            entity.direction + dir_vec
        } else {
            entity.direction - dir_vec
        };
        entity.direction = new_dir;
        // update the angle based on the last active direction
        if new_dir.len() > 0.0 {
            let new_angle = if new_dir.y >= 0.0 {
                new_dir.angle()
            } else {
                new_dir.rotate(PI).angle() + PI
            };
            // + pi because drawing is upside down
            entity.angle = new_angle + PI;
        } else {
            // no direction do not change angle
        }
    }

    pub(crate) fn shoot(&mut self) {
        let id = self.lander.entity_id;
        let entity = self.get_entity_immutable(id);
        let position = entity.position;
        let init_velocity = 40.0 * (entity.velocity() + 1.0);
        let direction = Vec2d::from_angle(entity.angle) * -1.0 * init_velocity;
        self.sound.shoot();
        self.create_missile(position, direction);
    }

    fn missile_tick(&mut self, time_in_ms: f32) {
        for missile in self.missiles.iter_mut() {
            missile.time_to_live -= time_in_ms / 1000.0f32;
        }
    }

    fn do_collision_detection(&mut self) {
        let id = self.lander.entity_id;
        let lander_entity = self.get_entity_immutable(id);
        let lander_position = lander_entity.position;

        let mut asteroids_to_delete = Vec::<Asteroid>::new();
        let mut missiles_to_delete = Vec::<usize>::new();

        for ast in self.asteroids.iter() {
            // Check collision against player:
            let asteroid_entity = self.get_entity_immutable(ast.entity_id);
            let asteroid_hull = &ast.get_transformed_hull(asteroid_entity);
            let collision = collision::hit_test(lander_position, asteroid_hull); // Primitive! This will only ever trigger, if the center of the starship is inside the asteroid.
            if collision {
                self.sound.die();
                self.game_state = State::Lost;
            }

            // Check collision against missiles
            for m in self.missiles.iter() {
                let missile_entity = self.get_entity_immutable(m.entity_id);
                let projectile_collision =
                    collision::hit_test(missile_entity.position, asteroid_hull);

                // create two new asteroids, smaller than the previous one, but
                // flying in other directions.
                // also: Schedule this asteroid for deletion
                if projectile_collision {
                    self.sound.explode();
                    asteroids_to_delete.push(ast.clone());
                    missiles_to_delete.push(m.entity_id);
                    if ast.get_scale() == MAX_SCALE {
                        self.whiteout_frames = 5;
                        self.screen_shake_frames = 25;
                        self.screen_shake_strength = 12.0;
                    }
                    self.score += 100;
                }
            }
        }

        let new_missiles = self
            .missiles
            .clone()
            .into_iter()
            .filter(|a| !missiles_to_delete.contains(&a.entity_id));
        self.garbage_collect_entities(&missiles_to_delete);
        self.missiles = new_missiles.collect();
    }

    fn renderHud(&mut self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        let id = self.lander.entity_id;
        let entity = self.get_entity(id);
        let position = entity.position;
        let direction = entity.direction;
        self.hud
            .update(position, direction, self.score, self.asteroids.len() as u32);
        self.hud.render(canvas);
    }

    fn render_starship(
        &self,
        lander_entity: &Entity,
        screen_space_transform: TransformationMatrix,
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
        textures: &HashMap<String, Texture>,
    ) {
        let scale =
            vecmath::TransformationMatrix::scale(graphics::LanderScale.x, graphics::LanderScale.y);
        let entity_trans = lander_entity.get_screenspace_transform(screen_space_transform);
        // fix orientation of lander and rotate 90 deg
        let offset = vecmath::TransformationMatrix::rotate(PI / 2.0);
        let transform = entity_trans * scale * offset;
        let items = [&graphics::StarShip];
        let texture = textures.get("neon").unwrap();
        for lander_part in items.iter() {
            let geometry = transform.transform_many(&lander_part.to_vec());
            draw::neon_draw_lines(canvas, &geometry, Color::RGB(128, 255, 255), true, texture)
                .unwrap();
        }

        if self.lander.drive_enabled {
            let geometry;
            geometry = transform.transform_many(&graphics::FlameA.to_vec());
            draw::draw_lines(canvas, &geometry, Color::RGB(255, 255, 255), true).unwrap();
        }
    }

    fn render_starfield(
        &self,
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
        textures: &HashMap<String, Texture>,
    ) {
        let lander_pos = self
            .get_entity_immutable(self.lander.entity_id)
            .position
            .clone();

        let texture = textures.get("star").unwrap();
        for star in self.starfield.iter() {
            let starpos = (star.pos.clone() - lander_pos) * (0.75 + star.layer as f32) as f32;
            let _ = canvas.copy(
                texture,
                None,
                sdl2::rect::Rect::new(
                    starpos.x as i32 % WorldSize.x as i32,
                    starpos.y as i32 % WorldSize.x as i32,
                    12 / (1 + star.layer as u32),
                    12 / (1 + star.layer as u32),
                ),
            );
        }
    }

    fn make_starfield() -> Vec<Star> {
        let mut output = Vec::<Star>::new();
        let mut rnd = rand::thread_rng();
        for _ in 0..3000 {
            let s = Star {
                pos: Vec2d::new(
                    rnd.gen_range(0..WorldSize.x as i32) as f32,
                    rnd.gen_range(0..WorldSize.y as i32) as f32,
                ),
                layer: rnd.gen_range(0..=3) as u8,
            };
            output.push(s);
        }
        output
    }

    pub fn toggle_background_music(&mut self) {
        self.sound.toggle_background_music();
    }
}

mod tests {
    use crate::{simulation, vecmath::Vec2d};

    use super::{Entity, Physics};
}
