use core::f32;
use std::collections::HashMap;
use std::f32::consts::PI;

use rand::{thread_rng, Rng};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Texture};

use crate::graphics::{
    self, render_game_over, ENTITY_SCALE, MINIRECT_ENEMY, MISSILE, RECT_ENEMY, ROMBUS_ENEMY,
    STARSHIP_COLOR,
};
use crate::sound;
use crate::vecmath::TransformationMatrix;
use crate::{
    collision, draw, hud,
    vecmath::{self, Vec2d},
};

use self::enemy::{Enemy, EnemyType};
use self::entity::Entity;
use self::explosion::Explosion;

mod enemy;
mod entity;
mod explosion;
mod objectstore;
mod vertex;
mod vertexgrid;

use self::objectstore::{ObjectDefault, ObjectStore};
use self::vertexgrid::VertexGrid;

const MAX_ACCELERATION: f32 = 800.0;
const VELOCITY_SPACESHIP: f32 = 450.0;
const VELOCITY_MISSILE: f32 = 800.0;

const MAX_SHOOT_COOLDOWN: f32 = 0.15;
const MIN_SHOOT_COOLDOWN: f32 = 0.08;

const NUM_EXPLOSION_FARMES: u32 = 50;

#[derive(Clone, PartialEq, Debug)]
pub enum BorderBehavior {
    Dismiss,
    Bounce,
    BounceSlowdown,
}

pub const BIT_LEFT: u16 = 0b1;
pub const BIT_RIGHT: u16 = 0b10;
pub const BIT_UP: u16 = 0b100;
pub const BIT_DOWN: u16 = 0b1000;
pub const BIT_SHOOT_LEFT: u16 = 0b10000;
pub const BIT_SHOOT_RIGHT: u16 = 0b100000;
pub const BIT_SHOOT_UP: u16 = 0b1000000;
pub const BIT_SHOOT_DOWN: u16 = 0b10000000;
pub const MOVEMENT_MASK: u16 = BIT_LEFT | BIT_RIGHT | BIT_UP | BIT_DOWN;

pub struct Starship {
    entity_id: usize,
    drive_enabled: bool,
    shoot_direction: Vec2d,
    shoot_cooldown: f32,       // expected cooldown time (sec)
    shoot_cooldown_count: f32, // current cooldown value (sec)
}

#[derive(Clone, PartialEq, Default)]
pub struct Missile {
    entity_id: usize,
    time_to_live: f32, // in seconds
}

impl ObjectDefault for Missile {
    fn default() -> Self {
        Missile {
            entity_id: 0,
            time_to_live: 5.0f32,
        }
    }
}

#[derive(Clone)]
pub struct FloatingText {
    position: Vec2d,
    time_to_live: f32, // in seconds
    text: String,
}
impl FloatingText {
    fn new(position: Vec2d, text: String) -> Self {
        Self {
            position: position,
            time_to_live: 1.0,
            text: text,
        }
    }
}

#[derive(PartialEq)]
pub enum State {
    Running,
    Lost,
}

pub struct World {
    game_control_bits: u16,
    entities: ObjectStore<Entity>,
    missiles: ObjectStore<Missile>,
    grid: VertexGrid,
    enemies: Vec<Enemy<'static>>,
    texts: Vec<FloatingText>,
    explosions: Vec<Explosion>,
    starship: Starship,
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
    x: 1.5 * 800.0,
    y: 1.5 * 600.0,
};

const GRID_DISTANCE: f32 = 20.0;

impl Missile {
    pub fn new(id: usize) -> Self {
        Self {
            entity_id: id,
            time_to_live: 5.0f32,
        }
    }
}

impl World {
    pub fn new(window_width: u32, window_height: u32) -> Self {
        let lander = Starship {
            entity_id: 0,
            drive_enabled: false,
            shoot_direction: Vec2d::default(),
            shoot_cooldown: MAX_SHOOT_COOLDOWN,
            shoot_cooldown_count: 0.0,
        };

        let mut store: ObjectStore<Entity> = ObjectStore::<Entity>::new();
        let lander_id = store.create_object();

        let mut lander_entity = store.get_object_clone(lander_id);
        lander_entity.set_position(WORLD_SIZE / 2.0);
        lander_entity.set_max_velocity(VELOCITY_SPACESHIP);
        lander_entity.set_border_behavior(BorderBehavior::BounceSlowdown);
        store.update_object(lander_id, lander_entity);

        let w = World {
            game_control_bits: 0,
            entities: store,
            starship: lander,
            enemies: Vec::new(),
            texts: Vec::new(),
            explosions: Vec::new(),
            hud: hud::Hud::new(),
            game_state: State::Running,
            missiles: ObjectStore::new(),
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
        return self.entities.create_object();
    }

    pub fn create_missile(&mut self, pos: Vec2d, direction: Vec2d) {
        let id = self.entities.create_object();
        let mut entity = self.entities.get_object_clone(id);
        entity.set_position(pos);
        entity.set_direction(direction);
        entity.set_acceleration(direction * MAX_ACCELERATION);
        entity.set_max_velocity(VELOCITY_MISSILE);
        entity.set_border_behavior(BorderBehavior::Dismiss);
        entity.set_angle(direction.angle_360());
        self.entities.update_object(id, entity);
        self.missiles.insert_object(Missile::new(id));
    }

    fn garbage_collect_entities(&mut self, ids_to_remove: &Vec<usize>) {
        self.entities.garbage_collect(ids_to_remove);
    }

    pub fn dismiss_dead_missiles(&mut self) {
        // Wrong: We must not remove the missile ids from the entities!
        let entities_to_remove = self.missiles.filter_map(|x| {
            if x.time_to_live <= 0.0 {
                Some(x.entity_id)
            } else {
                None
            }
        });
        self.garbage_collect_entities(&entities_to_remove);
        self.missiles
            .garbage_collect_filter(|m| m.time_to_live <= 0.0);
    }

    fn texts_tick(&mut self, time_in_ms: f32) {
        for txt in self.texts.iter_mut() {
            txt.time_to_live -= time_in_ms / 1000.0f32;
        }
        self.texts.retain(|t| t.time_to_live > 0.0);
    }

    fn explosion_tick(&mut self) {
        for exp in &mut self.explosions {
            exp.frame_count += 2;
        }
        self.explosions
            .retain(|e| e.frame_count < NUM_EXPLOSION_FARMES);
    }

    pub fn apply_control(&mut self) {
        self.entities
            .with(self.starship.entity_id, |e: &mut Entity| {
                // shooting:
                self.starship.shoot_direction = Vec2d::default();
                let mut new_shoot_dir = Vec2d::default();
                if self.game_control_bits & BIT_SHOOT_LEFT != 0 {
                    new_shoot_dir = new_shoot_dir + Vec2d { x: -1.0, y: 0.0 };
                }

                if self.game_control_bits & BIT_SHOOT_RIGHT != 0 {
                    new_shoot_dir = new_shoot_dir + Vec2d { x: 1.0, y: 0.0 };
                }

                if self.game_control_bits & BIT_SHOOT_UP != 0 {
                    new_shoot_dir = new_shoot_dir + Vec2d { x: 0.0, y: -1.0 };
                }

                if self.game_control_bits & BIT_SHOOT_DOWN != 0 {
                    new_shoot_dir = new_shoot_dir + Vec2d { x: 0.0, y: 1.0 };
                }
                self.starship.shoot_direction = new_shoot_dir;

                // Movement:
                // Nothing set, break immediately!
                if self.game_control_bits & MOVEMENT_MASK == 0 {
                    e.set_direction(Vec2d::default());
                    e.set_acceleration(Vec2d::default());
                    self.starship.drive_enabled = false;
                    return;
                }

                let mut new_dir = Vec2d::default();
                if self.game_control_bits & BIT_LEFT != 0 {
                    new_dir = new_dir + Vec2d { x: -1.0, y: 0.0 };
                }

                if self.game_control_bits & BIT_RIGHT != 0 {
                    new_dir = new_dir + Vec2d { x: 1.0, y: 0.0 };
                }

                if self.game_control_bits & BIT_UP != 0 {
                    new_dir = new_dir + Vec2d { x: 0.0, y: -1.0 };
                }

                if self.game_control_bits & BIT_DOWN != 0 {
                    new_dir = new_dir + Vec2d { x: 0.0, y: 1.0 };
                }
                new_dir = new_dir * MAX_ACCELERATION;
                e.set_direction(e.direction() + new_dir);

                let dir_vec = e.direction();
                let accel_factor = dir_vec * MAX_ACCELERATION;
                e.set_acceleration(accel_factor);

                self.starship.drive_enabled = e.direction().is_not_zero();

                if accel_factor.len() > 0.0 {
                    let new_angle = accel_factor.angle_360();
                    // + pi because drawing is upside down
                    e.set_angle(new_angle + PI);
                } else {
                    // no direction do not change angle
                }
            })
    }

    pub fn tick(&mut self, time_in_ms: f32, tick_resolution_in_ms: f32) {
        if self.game_state != State::Running {
            return;
        }

        let sim_time_in_seconds = time_in_ms / 1000.0;
        let mut num_ticks = (sim_time_in_seconds / tick_resolution_in_ms) as usize;
        if num_ticks == 0 {
            num_ticks = 1;
        }

        self.apply_control();

        self.entities
            .for_each(|e: &mut Entity, _: usize| e.physics_tick(sim_time_in_seconds, num_ticks));

        self.missile_tick(time_in_ms);
        self.dismiss_dead_missiles();
        self.enemy_tick();
        self.texts_tick(time_in_ms);
        self.explosion_tick();
        self.grid_tick();

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

        let lander_entity = self.entities.get_object(self.starship.entity_id);

        let mut screen_space_transform = TransformationMatrix::unit();
        screen_space_transform = screen_space_transform
            * TransformationMatrix::translation_v(lander_entity.position() * -1.0)
            * shake_transform
            * TransformationMatrix::translation_v(self.screen_size / 2.0); // center to screen

        self.render_grid(canvas, screen_space_transform); //render gris first
        self.render_world_border(canvas, screen_space_transform);
        self.render_enemies(canvas, screen_space_transform, textures);
        self.render_explosions(canvas, screen_space_transform, textures);
        self.render_texts(canvas, screen_space_transform);
        self.render_starship(&lander_entity, screen_space_transform, canvas, textures);
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
        self.missiles.for_each(|missile, _| {
            let entity = self.entities.get_object(missile.entity_id);
            let scale = vecmath::TransformationMatrix::scale(7f32, 7f32);
            let entity_trans = entity.get_screenspace_transform(screen_space_transform) * scale;
            let vecs = entity_trans.transform_many(&MISSILE.to_vec());
            let _ = draw::draw_lines(canvas, &vecs, Color::RGBA(255, 255, 255, 255), true);
        });
    }

    pub(crate) fn update_window_size(&mut self, width: f32, height: f32) {
        self.screen_size.x = width;
        self.screen_size.y = height;
    }

    pub(crate) fn modify_control_bit(&mut self, dir: u16, enable: bool) {
        if self.game_state != State::Running {
            return;
        }

        if enable {
            self.game_control_bits |= dir;
        } else {
            self.game_control_bits &= !dir;
        }
    }

    fn missile_tick(&mut self, time_in_ms: f32) {
        if self.starship.shoot_cooldown_count > 0.0 {
            self.starship.shoot_cooldown_count -= time_in_ms / 1000.0;
        }

        if self.starship.shoot_direction.len() > 0.0 && self.starship.shoot_cooldown_count <= 0.0 {
            self.starship.shoot_cooldown_count = self.starship.shoot_cooldown;
            let id = self.starship.entity_id;
            let lander_entity = self.entities.get_object(id);
            let position = lander_entity.position();
            let init_velocity = 40.0 * (lander_entity.velocity() + 1.0);
            let direction = self.starship.shoot_direction * init_velocity;
            self.sound.shoot();
            self.create_missile(position, direction);
        }

        self.missiles.for_each(|missile, _| {
            missile.time_to_live -= time_in_ms / 1000.0f32;

            let id = missile.entity_id;
            let entity = self.entities.get_object(id);
            let position = entity.position();
            Self::intersect_grid(position, &mut self.grid);
        });
    }

    fn intersect_grid(missile_pos: Vec2d, grid: &mut VertexGrid) {
        grid.intersect_grid(missile_pos);
    }

    fn make_safe_enemy_position(&self) -> Vec2d {
        loop {
            let pos = Vec2d {
                x: thread_rng().gen_range(0..(WORLD_SIZE.x as usize)) as f32,
                y: thread_rng().gen_range(0..(WORLD_SIZE.y as usize)) as f32,
            };
            let id = self.starship.entity_id;
            let player_pos = self.entities.get_object(id).position();
            if (player_pos - pos).len() > 300f32 {
                return pos;
            }
        }
    }

    fn enemy_tick(&mut self) {
        // check if we have enough enemies:
        self.spawn_enemies();

        for i in 0..self.enemies.len() {
            let en = &self.enemies[i];
            en.tick(&self.entities, self.starship.entity_id, &self.missiles);
        }
    }

    fn spawn_enemies(&mut self) {
        if self.enemies.len() < 50 {
            let should_spawn = thread_rng().gen_ratio(1, 100);

            if should_spawn {
                let num_to_spawn = thread_rng().gen_range(2..10);

                for _ in 0..num_to_spawn {
                    let ent = self.create_entity();
                    let enemy;

                    let enemy_type = thread_rng().gen_range(0..EnemyType::Invalid as usize);

                    match enemy_type {
                        0 => {
                            enemy = Enemy {
                                ty: EnemyType::Rombus,
                                entity_id: ent,
                                hull: &ROMBUS_ENEMY,
                            }
                        }
                        1 => {
                            enemy = Enemy {
                                ty: EnemyType::Rect,
                                entity_id: ent,
                                hull: &RECT_ENEMY,
                            }
                        }
                        2 => {
                            enemy = Enemy {
                                ty: EnemyType::Wanderer,
                                entity_id: ent,
                                hull: &RECT_ENEMY,
                            }
                        }
                        3 => {
                            enemy = Enemy {
                                ty: EnemyType::SpawningRect,
                                entity_id: ent,
                                hull: &RECT_ENEMY,
                            }
                        }
                        4 => {
                            // We don't spawn minirects directly
                            enemy = Enemy {
                                ty: EnemyType::SpawningRect,
                                entity_id: ent,
                                hull: &MINIRECT_ENEMY,
                            }
                        }
                        _ => {
                            panic!("Bad enemy type!");
                        }
                    }

                    let epos = self.make_safe_enemy_position();
                    self.entities.with(ent, |the_entity| {
                        the_entity.set_position(epos);
                        the_entity.set_position(epos);
                        the_entity.set_border_behavior(BorderBehavior::Bounce);
                    });
                    self.enemies.push(enemy);
                }
            }
        }
    }

    fn grid_tick(&mut self) {
        self.grid.tick();
    }

    fn do_collision_detection(&mut self) {
        let id = self.starship.entity_id;
        let lander_entity = self.entities.get_object(id);
        let lander_position = lander_entity.position();

        let mut enemies_to_delete = Vec::<usize>::new();
        let mut missiles_to_delete = Vec::<usize>::new();
        let mut minirect_spawns = Vec::<usize>::new();

        let mut new_hit_points: u32 = 0;
        let mut new_texts: Vec<FloatingText> = Vec::new();
        for enemy in self.enemies.iter() {
            // create collidable hull for entity:
            let enemy_ent = self.entities.get_object(enemy.entity_id);
            let enemy_pos = enemy_ent.position();
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
            self.missiles.for_each(|missile, _| {
                let missile_entity = self.entities.get_object(missile.entity_id);
                let projectile_collision =
                    collision::hit_test(missile_entity.position(), &enemy_hull);

                // create two new asteroids, smaller than the previous one, but
                // flying in other directions.
                // also: Schedule this asteroid for deletion
                if projectile_collision {
                    self.sound.explode();
                    enemies_to_delete.push(enemy.entity_id);

                    if !missiles_to_delete.contains(&missile.entity_id) {
                        missiles_to_delete.push(missile.entity_id);
                    }

                    new_hit_points += enemy.get_score();
                    if enemy.ty == EnemyType::SpawningRect {
                        minirect_spawns.push(missile.entity_id);
                    }
                    new_texts.push(FloatingText::new(
                        enemy_pos,
                        format!("{}", enemy.get_score()),
                    ));
                    self.explosions.push(Explosion::new(enemy_pos));
                }
            });
        }
        self.update_score(new_hit_points);
        self.texts.extend(new_texts);

        self.spawn_minirects(minirect_spawns);

        self.missiles
            .garbage_collect_filter(|x| missiles_to_delete.contains(&x.entity_id));

        let new_enemies = self
            .enemies
            .clone()
            .into_iter()
            .filter(|a| !enemies_to_delete.contains(&a.entity_id));

        self.garbage_collect_entities(&missiles_to_delete);
        self.garbage_collect_entities(&enemies_to_delete);
        self.enemies = new_enemies.collect();
    }

    fn spawn_minirects(&mut self, minirect_spawns: Vec<usize>) {
        for id in minirect_spawns.iter() {
            let spawnpos;
            {
                let entity = self.entities.get_object(*id);
                spawnpos = entity.position();
            }

            for i in 0..2 {
                let new_ent_id = self.create_entity();
                let mut new_ent = self.entities.get_object_clone(new_ent_id);
                new_ent.set_position(spawnpos.clone() + Vec2d::new(16f32, 16f32) * i as f32);
                new_ent.set_border_behavior(BorderBehavior::Bounce);
                let minirect = Enemy {
                    entity_id: new_ent_id,
                    ty: EnemyType::MiniRect,
                    hull: &MINIRECT_ENEMY,
                };
                self.entities.update_object(new_ent_id, new_ent);
                self.enemies.push(minirect);
            }
        }
    }

    fn update_score(&mut self, hit_points: u32) {
        let new_score = self.score + hit_points;
        const COOLDOWN_STEP: f32 = (MAX_SHOOT_COOLDOWN - MIN_SHOOT_COOLDOWN) / 3.0;
        for (level_score, shoot_cooldown_idx) in vec![(5_000, 1), (10_000, 2), (15_000, 3)] {
            if self.score <= level_score && level_score < new_score {
                self.starship.shoot_cooldown =
                    MAX_SHOOT_COOLDOWN - (shoot_cooldown_idx as f32 * COOLDOWN_STEP);
            }
        }
        self.score = new_score;
    }

    fn render_hud(&mut self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        let id = self.starship.entity_id;
        let entity = self.entities.get_object(id);
        let position = entity.position();
        let direction = entity.direction();
        let acceleration = entity.acceleration();
        let angle = entity.angle();
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

        if self.starship.drive_enabled {
            let geometry;
            geometry = transform.transform_many(&graphics::FLAME_A.to_vec());
            draw::draw_lines(canvas, &geometry, Color::RGB(255, 255, 255), true).unwrap();
        }

        let lander_center = screen_space_transform.transform(&lander_entity.position());
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
            let entity = self.entities.get_object(enemy.entity_id);
            enemy.render(canvas, screen_space_transform, textures, &entity);
        }
    }

    fn render_explosions(
        &self,
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
        screen_space_transform: TransformationMatrix,
        textures: &HashMap<String, Texture<'_>>,
    ) {
        let texture = textures.get("star").unwrap();
        for exp in &self.explosions {
            let pos = exp.position;
            let pos_screen = screen_space_transform.transform(&pos);
            for sparc_dir in &exp.sparc_dir {
                let pos_screen = pos_screen + (sparc_dir.clone() * exp.frame_count as f32);

                _ = canvas.copy(
                    texture,
                    None,
                    sdl2::rect::Rect::new(pos_screen.x as i32, pos_screen.y as i32, 12, 12),
                );
            }
        }
    }

    fn render_texts(
        &self,
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
        screen_space_transform: TransformationMatrix,
    ) {
        for text_ele in &self.texts {
            let screen_coordinate = screen_space_transform.transform(&text_ele.position);
            let screen_coordinate =
                Point::new(screen_coordinate.x as i32, screen_coordinate.y as i32);
            draw::draw_text(
                canvas,
                &text_ele.text,
                15,
                screen_coordinate,
                Color::RGB(0, 255, 0),
            )
            .unwrap();
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

        let _ = draw::draw_line(canvas, &top_left, &top_right, Color::WHITE);
        let _ = draw::draw_line(canvas, &top_left, &bot_left, Color::WHITE);
        let _ = draw::draw_line(canvas, &top_right, &bot_right, Color::WHITE);
        let _ = draw::draw_line(canvas, &bot_left, &bot_right, Color::WHITE);
    }

    fn render_grid(
        &self,
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
        screen_space_transform: TransformationMatrix,
    ) {
        self.grid.render(canvas, screen_space_transform);
    }

    fn make_grid() -> VertexGrid {
        VertexGrid::new()
    }

    pub fn toggle_background_music(&mut self) {
        self.sound.toggle_background_music();
    }

    // pub fn missiles(&self) -> &[Missile] {
    //     &self.missiles
    // }
}
