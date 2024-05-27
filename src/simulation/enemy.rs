use std::collections::HashMap;

use rand::{thread_rng, Rng};
use sdl2::render::Texture;

use crate::{
    draw,
    graphics::{
        self, MINIRECT_ENEMY, MINIRECT_ENEMY_COLOR, RECT_ENEMY, RECT_ENEMY_COLOR, ROMBUS_ENEMY,
        ROMBUS_ENEMY_COLOR, WANDERER_ENEMY, WANDERER_ENEMY_COLOR,
    },
    vecmath::{self, TransformationMatrix, Vec2d},
};

use super::{entity::Entity, objectstore::ObjectStore, Missile};

#[derive(Clone, Copy, PartialEq)]
pub enum EnemyType {
    Rect,
    Rombus,
    Wanderer,
    SpawningRect,
    MiniRect,
    Invalid,
}

#[derive(Clone)]
pub struct Enemy<'a> {
    pub entity_id: usize,
    pub ty: EnemyType,
    pub hull: &'a [Vec2d],
}

impl Enemy<'_> {
    pub fn get_score(&self) -> u32 {
        return match self.ty {
            EnemyType::Invalid => 0,
            EnemyType::Rombus => 100,
            EnemyType::Rect => 300,
            EnemyType::Wanderer => 200,
            EnemyType::SpawningRect => 200,
            EnemyType::MiniRect => 50,
        };
    }

    pub fn tick(
        &self,
        entities: &ObjectStore<Entity>,
        player_id: usize,
        missiles: &ObjectStore<Missile>,
    ) {
        let ty = self.ty;
        let playerpos = entities.get_object(player_id).position().clone();

        match ty {
            EnemyType::Rect => self.rect_tick(entities, playerpos, missiles),
            EnemyType::Rombus => self.rombus_tick(entities, playerpos),
            EnemyType::Wanderer => self.wanderer_tick(entities),
            EnemyType::SpawningRect => self.rect_tick(entities, playerpos, missiles),
            EnemyType::MiniRect => self.rect_tick(entities, playerpos, missiles),
            _ => {}
        }
    }

    pub fn render(
        &self,
        canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
        screen_space_transform: TransformationMatrix,
        textures: &HashMap<String, Texture<'_>>,
        source_entity: &Entity,
    ) {
        let items: &[Vec2d];
        let col;
        match self.ty {
            EnemyType::Rect => {
                items = &RECT_ENEMY;
                col = RECT_ENEMY_COLOR;
            }
            EnemyType::Rombus => {
                items = &ROMBUS_ENEMY;
                col = ROMBUS_ENEMY_COLOR;
            }
            EnemyType::Wanderer => {
                items = &WANDERER_ENEMY;
                col = WANDERER_ENEMY_COLOR
            }
            EnemyType::SpawningRect => {
                items = &RECT_ENEMY;
                col = MINIRECT_ENEMY_COLOR;
            }
            EnemyType::MiniRect => {
                items = &MINIRECT_ENEMY;
                col = MINIRECT_ENEMY_COLOR;
            }
            EnemyType::Invalid => todo!(),
        }
        let scale = vecmath::TransformationMatrix::scale(
            graphics::ENTITY_SCALE.x,
            graphics::ENTITY_SCALE.y,
        );
        let entity_trans = source_entity.get_screenspace_transform(screen_space_transform) * scale;
        let texture = textures.get("neon").unwrap();
        let geometry = entity_trans.transform_many(&items.to_vec());
        draw::neon_draw_lines(canvas, &geometry, col, true, texture).unwrap();
    }

    fn rombus_tick(&self, world: &ObjectStore<Entity>, player_pos: Vec2d) {
        world.with(self.entity_id, |ent| {
            const ROMBUS_VEL: f32 = 120.0f32;
            let new_dir = (player_pos - ent.position()).normalized() * ROMBUS_VEL;
            ent.set_acceleration(new_dir);
            ent.set_direction(new_dir);
            ent.set_max_velocity(ROMBUS_VEL);
        });
    }

    fn rect_tick(
        &self,
        world: &ObjectStore<Entity>,
        player_pos: Vec2d,
        missiles: &ObjectStore<Missile>,
    ) {
        let vel = match self.ty {
            EnemyType::Rect => 120f32,
            EnemyType::Rombus => todo!(),
            EnemyType::Wanderer => todo!(),
            EnemyType::SpawningRect => 100f32,
            EnemyType::MiniRect => 250f32,
            EnemyType::Invalid => todo!(),
        };
        let current_pos;
        let mut new_dir;
        {
            current_pos = world.get_object(self.entity_id).position();
            new_dir = (player_pos - current_pos).normalized() * vel;
        }

        missiles.for_each(|missile, _| {
            let missile_ent = world.get_object(missile.entity_id);
            let missile_pos = missile_ent.position();
            // each missile will apply a force on the rect enemy, that
            // is inversely proportional to the distance
            let missile_dist = current_pos - missile_pos;
            const FORCE_RANGE: f32 = 128f32;
            let missile_dist_units = missile_dist.len();
            let relative_force_strength = 1.0f32 - (missile_dist_units / FORCE_RANGE);

            if relative_force_strength > 1.0f32 || relative_force_strength < 0.0f32 {
                return;
            }

            new_dir = new_dir + ((missile_dist.normalized()) * relative_force_strength * vel);
        });

        world.with(self.entity_id, |ent| {
            ent.set_acceleration(new_dir);
            ent.set_direction(new_dir);
            ent.set_max_velocity(vel);
        });
    }

    fn wanderer_tick(&self, world: &ObjectStore<Entity>) {
        world.with(self.entity_id, |ent| {
            const MAX_VEL: f32 = 80f32;
            let new_dir = Vec2d {
                x: thread_rng().gen_range(-1.0..1.0) as f32,
                y: thread_rng().gen_range(-1.0..1.0) as f32,
            };
            ent.set_direction(new_dir.normalized() * MAX_VEL);
            ent.set_acceleration(ent.direction() * MAX_VEL);
            ent.set_max_velocity(MAX_VEL);
        });
    }
}
