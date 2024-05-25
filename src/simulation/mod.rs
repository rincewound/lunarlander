#![feature(drain_filter)]

use std::collections::HashMap;
use std::{f32::consts::PI, num};

use rand::{thread_rng, Rng};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Texture};

use crate::asteroids::{self, MAX_SCALE};
use crate::draw::draw_lines;
use crate::graphics::{
    self, render_game_over, render_won_text, ENTITY_SCALE, MISSILE, RECT_ENEMY, RECT_ENEMY_COLOR,
    ROMBUS_ENEMY, ROMBUS_ENEMY_COLOR, STARSHIP_COLOR,
};
use crate::sound;
use crate::vecmath::TransformationMatrix;
use crate::{
    asteroids::Asteroid,
    collision, draw, hud,
    vecmath::{self, Vec2d},
};

const MAX_ACCELERATION: f32 = 100.0;
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

impl DirectionKey {
    fn get_vec2d(&self) -> Vec2d {
        match self {
            DirectionKey::Up => Vec2d::new(0.0, -1.0),
            DirectionKey::Down => Vec2d::new(0.0, 1.0),
            DirectionKey::Left => Vec2d::new(-1.0, 0.0),
            DirectionKey::Right => Vec2d::new(1.0, 0.0),
        }
    }
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

#[derive(Clone, Copy)]
enum EnemyType {
    Rect,
    Rombus,
}

#[derive(Clone)]
pub struct Enemy<'a> {
    entity_id: usize,
    ty: EnemyType,
    hull: &'a [Vec2d],
}

#[derive(PartialEq)]
pub enum State {
    Running,
    Won,
    Lost,
}

pub struct World {
    p: Physics,
    next_entity_id: usize,
    entities: Vec<Entity>,
    missiles: Vec<Missile>,
    grid: Vec<Vertex>,
    enemies: Vec<Enemy<'static>>,
    lander: Lander,
    hud: hud::Hud,
    game_state: State,
    score: u32,

    screen_shake_frames: usize,
    screen_shake_strength: f32,
    screen_size: Vec2d,
    sound: sound::Sound,
    whiteout_frames: u32, // number of frames to draw white, when large asteroids are destroyed
}

const WORLD_SIZE: Vec2d = Vec2d {
    x: 1.0 * 800.0,
    y: 1.0 * 600.0,
};

const GRID_DISTANCE: f32 = 20.0;
pub struct Vertex {
    main_position: Vec2d,
    positoin: Vec2d,
}

impl Vertex {
    pub fn new(pos: Vec2d) -> Self {
        Self {
            positoin: pos,
            main_position: pos,
        }
    }
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

impl World {
    pub fn new(window_width: u32, window_height: u32) -> Self {
        let lander = Lander {
            entity_id: 0,
            drive_enabled: false,
        };

        let mut lander_entity = Entity::default(0);
        lander_entity.set_position(WORLD_SIZE / 2.0);
        lander_entity.max_velocity = VELOCITY_SPACESHIP;
        lander_entity.border_behavior = BorderBehavior::BounceSlowdown;

        let w = World {
            next_entity_id: 1,
            p: Physics::default(),
            entities: vec![lander_entity],
            lander,
            enemies: Vec::new(),
            hud: hud::Hud::new(),
            game_state: State::Running,
            missiles: vec![],
            grid: Self::make_grid(),
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
        entity.angle = direction.angle();
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
            .physics_tick(time_in_ms, tick_resolution_in_ms, &mut self.entities);

        //TODO: maybe use this to update lander angle smoothly
        //let rotation = self.lander.rotation;
        let mut lander_entity = self.get_entity(self.lander.entity_id);
        if lander_entity.acceleration.len() < 0.01 {
            lander_entity.direction = if lander_entity.velocity() > 0.01 {
                let sim_time_in_seconds = time_in_ms / 1000.0;
                let break_fragment = lander_entity.direction.normalized()
                    * 2.0
                    * MAX_ACCELERATION
                    * sim_time_in_seconds;
                lander_entity.direction - break_fragment
            } else {
                Vec2d::default()
            }
        }

        self.missile_tick(time_in_ms);
        self.dismiss_dead_missiles();
        self.enemy_tick(time_in_ms);

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

        let mut shake_transform = TransformationMatrix::unit();
        if self.screen_shake_frames > 0 {
            self.screen_shake_frames -= 1;
            let mut rnd = rand::thread_rng();
            shake_transform = shake_transform
                * TransformationMatrix::translation(
                    rnd.gen_range(0..self.screen_shake_strength as i32) as f32 / 2.0,
                    rnd.gen_range(0..self.screen_shake_strength as i32) as f32 / 2.0,
                )
        }

        let lander_entity = self.get_entity_immutable(self.lander.entity_id);

        let mut screen_space_transform = TransformationMatrix::unit();
        screen_space_transform = screen_space_transform
            * TransformationMatrix::translation_v(lander_entity.position * -1.0)
            * shake_transform
            * TransformationMatrix::translation_v(self.screen_size / 2.0); // center to screen

        //self.render_starfield(canvas, textures);
        //self.render_asteroids(screen_space_transform, canvas);
        self.render_grid(canvas, screen_space_transform); //render gris first
        self.render_world_border(canvas, screen_space_transform);
        self.render_enemies(canvas, screen_space_transform, textures);
        self.render_starship(lander_entity, screen_space_transform, canvas, textures);
        self.render_missiles(screen_space_transform, canvas);

        if self.whiteout_frames > 0 {
            self.whiteout_frames -= 1;
            let alpha = 255 * (1.0 - (10.0 / self.whiteout_frames as f32)) as u8;
            canvas.set_draw_color(Color::RGBA(255, 255, 255, alpha));
            canvas.set_blend_mode(BlendMode::Mul);
            let _ = canvas.fill_rect(Rect::new(0, 0, 800, 600));
            canvas.set_draw_color(Color::RGBA(255, 255, 255, 255));
            canvas.set_blend_mode(BlendMode::None);
        }

        self.render_hud(canvas);
    }

    fn render_missiles(
        &mut self,
        screen_space_transform: TransformationMatrix,
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    ) {
        for missile in self.missiles.iter() {
            let entity = self.get_entity_immutable(missile.entity_id);
            let scale = vecmath::TransformationMatrix::scale(7f32, 7f32);
            let entity_trans = entity.get_screenspace_transform(screen_space_transform) * scale;
            let vecs = entity_trans.transform_many(&MISSILE.to_vec());
            let _ = draw::draw_lines(canvas, &vecs, Color::RGBA(255, 255, 255, 255), true);
        }
    }

    pub(crate) fn update_window_size(&mut self, width: f32, height: f32) {
        self.screen_size.x = width;
        self.screen_size.y = height;
    }

    pub(crate) fn direction_toggle(&mut self, dir: DirectionKey, enable: bool) {
        if self.game_state != State::Running {
            return;
        }
        let dir_vec = dir.get_vec2d();
        let entity = self.get_entity(self.lander.entity_id);
        let accel_factor = dir_vec * MAX_ACCELERATION;
        let new_accel = if enable {
            entity.acceleration + accel_factor
        } else {
            entity.acceleration - accel_factor
        };
        entity.set_acceleration(new_accel);
        if new_accel.len() > 0.0 {
            let new_angle = if new_accel.y >= 0.0 {
                new_accel.angle()
            } else {
                new_accel.rotate(PI).angle() + PI
            };
            // + pi because drawing is upside down
            entity.angle = new_angle + PI;
        } else {
            // no direction do not change angle
        }
        self.lander.drive_enabled = new_accel.len() > 0.001;
    }

    pub(crate) fn shoot(&mut self, dir: DirectionKey, enable: bool) {
        if self.game_state != State::Running {
            return;
        }
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

    fn enemy_tick(&mut self, time_in_ms: f32) {
        // check if we have enough enemies:
        if self.enemies.len() < 10 {
            let ent = self.create_entity();
            let enemy;

            if thread_rng().gen_range(0..100) > 50 {
                enemy = Enemy {
                    ty: EnemyType::Rombus,
                    entity_id: ent,
                    hull: &ROMBUS_ENEMY,
                };
            } else {
                enemy = Enemy {
                    ty: EnemyType::Rect,
                    entity_id: ent,
                    hull: &RECT_ENEMY,
                };
            }
            let the_entity = self.get_entity(ent);
            the_entity.position = Vec2d {
                x: thread_rng().gen_range(0..(WORLD_SIZE.x as usize)) as f32,
                y: thread_rng().gen_range(0..(WORLD_SIZE.y as usize)) as f32,
            };
            self.enemies.push(enemy);
        }

        for i in 0..self.enemies.len() {
            let en = &self.enemies[i];
            let ty = &en.ty;
            match ty {
                EnemyType::Rect => self.rombus_tick(i),
                EnemyType::Rombus => self.rombus_tick(i),
            }
        }
    }

    fn rombus_tick(&mut self, rombus_id: usize) {
        let player_pos = self
            .get_entity_immutable(self.lander.entity_id)
            .position
            .clone();
        let en = &self.enemies[rombus_id];
        let own_entity = self.get_entity(en.entity_id);
        let new_dir = (player_pos - own_entity.position).normalized() * 5.0f32;
        own_entity.acceleration = new_dir;
        own_entity.direction = new_dir;
        own_entity.max_velocity = 25.0f32;
    }

    fn do_collision_detection(&mut self) {
        let id = self.lander.entity_id;
        let lander_entity = self.get_entity_immutable(id);
        let lander_position = lander_entity.position;

        let mut enemies_to_delete = Vec::<usize>::new();
        let mut missiles_to_delete = Vec::<usize>::new();

        for enemy in self.enemies.iter() {
            // create collidable hull for entity:
            let enemy_ent = self.get_entity_immutable(enemy.entity_id);
            let enemy_transform = enemy_ent.get_transform();
            let scale_transform =
                vecmath::TransformationMatrix::scale(ENTITY_SCALE.x, ENTITY_SCALE.y);
            let hull_transform = enemy_transform * scale_transform;
            let enemy_hull = hull_transform.transform_many(&enemy.hull.to_vec());
            let collision = collision::hit_test(lander_position, &enemy_hull); // Primitive! This will only ever trigger, if the center of the starship is inside the asteroid.
            if collision {
                self.sound.die();
                self.game_state = State::Lost;
            }
            // Check collision against missiles
            for m in self.missiles.iter() {
                let missile_entity = self.get_entity_immutable(m.entity_id);
                let projectile_collision =
                    collision::hit_test(missile_entity.position, &enemy_hull);

                // create two new asteroids, smaller than the previous one, but
                // flying in other directions.
                // also: Schedule this asteroid for deletion
                if projectile_collision {
                    self.sound.explode();
                    enemies_to_delete.push(enemy.entity_id);
                    missiles_to_delete.push(m.entity_id);
                    self.score += 100;
                }
            }
        }

        let new_missiles = self
            .missiles
            .clone()
            .into_iter()
            .filter(|a| !missiles_to_delete.contains(&a.entity_id));
        let new_enemies = self
            .enemies
            .clone()
            .into_iter()
            .filter(|a| !enemies_to_delete.contains(&a.entity_id));
        self.garbage_collect_entities(&missiles_to_delete);
        self.garbage_collect_entities(&enemies_to_delete);
        self.missiles = new_missiles.collect();
        self.enemies = new_enemies.collect();
    }

    fn render_hud(&mut self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        let id = self.lander.entity_id;
        let entity = self.get_entity(id);
        let position = entity.position;
        let direction = entity.direction;
        let acceleration = entity.acceleration;
        let angle = entity.angle;
        self.hud.update(
            position,
            direction,
            acceleration,
            angle,
            self.score,
            self.enemies.len() as u32,
        );
        self.hud.render(canvas);
    }

    fn render_starship(
        &self,
        lander_entity: &Entity,
        screen_space_transform: TransformationMatrix,
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
        textures: &HashMap<String, Texture>,
    ) {
        let scale = vecmath::TransformationMatrix::scale(
            graphics::ENTITY_SCALE.x,
            graphics::ENTITY_SCALE.y,
        );
        let entity_trans = lander_entity.get_screenspace_transform(screen_space_transform);
        // fix orientation of lander and rotate 90 deg
        let offset = vecmath::TransformationMatrix::rotate(PI / 2.0);
        let transform = entity_trans * scale * offset;
        let items = [&graphics::STARSHIP];
        let texture = textures.get("neon").unwrap();
        for lander_part in items.iter() {
            let geometry = transform.transform_many(&lander_part.to_vec());
            draw::neon_draw_lines(canvas, &geometry, STARSHIP_COLOR, true, texture).unwrap();
        }

        if self.lander.drive_enabled {
            let geometry;
            geometry = transform.transform_many(&graphics::FLAME_A.to_vec());
            draw::draw_lines(canvas, &geometry, Color::RGB(255, 255, 255), true).unwrap();
        }

        let lander_center = screen_space_transform.transform(&lander_entity.position);
        let _ = canvas.copy(
            textures.get("halo").unwrap(),
            None,
            sdl2::rect::Rect::new(
                lander_center.x as i32 - 32,
                lander_center.y as i32 - 32,
                64,
                64,
            ),
        );
    }

    fn render_enemies(
        &self,
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
        screen_space_transform: TransformationMatrix,
        textures: &HashMap<String, Texture<'_>>,
    ) {
        for enemy in self.enemies.iter() {
            let entity = self.get_entity_immutable(enemy.entity_id);

            let items;
            let col;
            match enemy.ty {
                EnemyType::Rect => {
                    items = [&RECT_ENEMY];
                    col = RECT_ENEMY_COLOR;
                }
                EnemyType::Rombus => {
                    items = [&ROMBUS_ENEMY];
                    col = ROMBUS_ENEMY_COLOR;
                }
            }
            let scale = vecmath::TransformationMatrix::scale(
                graphics::ENTITY_SCALE.x,
                graphics::ENTITY_SCALE.y,
            );
            let entity_trans = entity.get_screenspace_transform(screen_space_transform) * scale;
            let texture = textures.get("neon").unwrap();
            for parts in items.iter() {
                let geometry = entity_trans.transform_many(&parts.to_vec());
                draw::neon_draw_lines(canvas, &geometry, col, true, texture).unwrap();
            }
        }
    }

    fn render_world_border(
        &self,
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
        screen_space_transform: TransformationMatrix,
    ) {
        let width: u32 = WORLD_SIZE.x as u32;
        let height: u32 = WORLD_SIZE.y as u32;
        let rect: Rect = sdl2::rect::Rect::new(0, 0, width, height);

        let mut top_left: Vec2d =
            Vec2d::new((rect.top_left()).x as f32, (rect.top_left()).y as f32);

        let mut top_right: Vec2d =
            Vec2d::new((rect.top_right()).x as f32, (rect.top_right()).y as f32);

        let mut bot_left: Vec2d =
            Vec2d::new((rect.bottom_left()).x as f32, (rect.bottom_left()).y as f32);

        let mut bot_right: Vec2d = Vec2d::new(
            (rect.bottom_right()).x as f32,
            (rect.bottom_right()).y as f32,
        );

        top_left = screen_space_transform.transform(&top_left);
        top_right = screen_space_transform.transform(&top_right);
        bot_left = screen_space_transform.transform(&bot_left);
        bot_right = screen_space_transform.transform(&bot_right);

        draw::draw_line(canvas, &top_left, &top_right, Color::WHITE);
        draw::draw_line(canvas, &top_left, &bot_left, Color::WHITE);
        draw::draw_line(canvas, &top_right, &bot_right, Color::WHITE);
        draw::draw_line(canvas, &bot_left, &bot_right, Color::WHITE);
    }

    fn render_grid(
        &self,
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
        screen_space_transform: TransformationMatrix,
    ) {
        let row_count: usize = ((WORLD_SIZE.x / GRID_DISTANCE) + 1.0) as usize;
        let col_count = ((WORLD_SIZE.y / GRID_DISTANCE) + 1.0) as usize;

        //draw horizontal
        let mut current_row: usize = 0;
        let mut j: usize = 0;
        while j < col_count {
            let mut i: usize = 0;
            while i < (row_count - 1) {
                let p1 = screen_space_transform
                    .transform(&self.grid[i + (current_row * row_count)].positoin);
                let p2: Vec2d = screen_space_transform
                    .transform(&self.grid[i + 1 + (current_row * row_count)].positoin);
                draw::draw_line(canvas, &p1, &p2, Color::BLUE);
                i += 1;
            }
            j += 1;
            current_row += 1;
        }

        //draw vertical
        let mut current_col: usize = 0;
        while current_col < row_count {
            let mut i: usize = 0;
            while i < (col_count - 1) {
                let p1 = screen_space_transform
                    .transform(&self.grid[i * row_count + current_col].positoin);
                let p2: Vec2d = screen_space_transform
                    .transform(&self.grid[(i + 1) * row_count + current_col].positoin);
                draw::draw_line(canvas, &p1, &p2, Color::BLUE);
                i += 1;
            }
            current_col += 1;
        }
    }

    fn make_grid() -> Vec<Vertex> {
        let mut grid: Vec<Vertex> = Vec::new();

        let mut y = 0.0;
        while y < WORLD_SIZE.y + 1.0 {
            let mut x = 0.0;
            while x < WORLD_SIZE.x + 1.0 {
                let pos: Vec2d = Vec2d::new(x, y);
                let vertex: Vertex = Vertex::new(pos);
                grid.push(vertex);
                x += GRID_DISTANCE;
            }
            y += GRID_DISTANCE;
        }
        return grid;
    }

    pub fn toggle_background_music(&mut self) {
        self.sound.toggle_background_music();
    }
}
