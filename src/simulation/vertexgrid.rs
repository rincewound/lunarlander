use sdl2::pixels::Color;

use crate::{
    draw,
    vecmath::{TransformationMatrix, Vec2d},
};

use super::{
    objectstore::{ObjectDefault, ObjectStore},
    vertex::Vertex,
    GRID_DISTANCE, WORLD_SIZE,
};

#[derive(Clone, Copy)]
struct CircularEffect {
    center: Vec2d,
    radius: f32,
    time_to_live: f32,
    expansion_speed: f32,
}

#[derive(Clone, Copy)]
enum Effect {
    Circular(CircularEffect),
}

impl ObjectDefault for Effect {
    fn default() -> Self {
        Effect::Circular(CircularEffect {
            center: Vec2d::default(),
            radius: 0.0,
            time_to_live: 0.0,
            expansion_speed: 0.0,
        })
    }
}

pub struct VertexGrid {
    grid: Vec<Vertex>,
    effects: ObjectStore<Effect>,
}

impl VertexGrid {
    pub fn new() -> Self {
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

        Self {
            grid,
            effects: ObjectStore::new(),
        }
    }

    pub fn add_circular_effect(
        &mut self,
        center: Vec2d,
        radius: f32,
        time_to_live: f32,
        expansion_speed: f32,
    ) {
        self.effects.insert_object(Effect::Circular(CircularEffect {
            center,
            radius,
            time_to_live,
            expansion_speed,
        }));
    }

    pub fn intersect_grid(&mut self, missile_pos: Vec2d) {
        let x = missile_pos.x;
        let y = missile_pos.y;

        if x > WORLD_SIZE.x || x < 0.0 || y < 0.0 || y > WORLD_SIZE.y {
            return;
        }

        let num_coll = (WORLD_SIZE.x / GRID_DISTANCE + 1.0) as usize;
        let w_off = (x / GRID_DISTANCE) as usize;
        let y_off = (y / GRID_DISTANCE) as usize;
        let index = y_off * num_coll + w_off;

        let mut indices: Vec<usize> = Vec::new();
        indices.push(index);
        if index > num_coll {
            indices.push(index - num_coll);
        }
        if index + num_coll < self.grid.len() {
            indices.push(index + num_coll);
        }
        if index % num_coll != 0 {
            indices.push(index + 1);
            indices.push(index - 1);
        }

        for &i in indices.iter() {
            let dir = missile_pos - self.grid[i].position();
            if dir.len() > 0.01 {
                self.grid[i].add_to_dir(dir.normalized() * 10.0);
            }
        }
    }

    pub fn apply_force(&mut self, force: Vec2d, pos: Vec2d, deltaT: f32) {
        let x = ((pos.x as i32) / GRID_DISTANCE as i32) as i32;
        let y = ((pos.y as i32) / GRID_DISTANCE as i32) as i32;

        if x > WORLD_SIZE.x as i32 || y > WORLD_SIZE.y as i32 || x < 0 || y < 0 {
            return;
        }

        let num_coll = (WORLD_SIZE.x / GRID_DISTANCE + 1.0) as i32;
        let index = (y * num_coll + x) as usize;
        // Stupid defensive programming here...
        if index < self.grid.len() {
            // clip max force to 500 units
            let mut next_dir = self.grid[index].direction() + force * deltaT;
            if next_dir.is_not_zero() {
                if next_dir.len() > 250.0 {
                    next_dir = next_dir.normalized() * 250.0;
                }
            }
            self.grid[index].add_to_dir(next_dir);
        }
    }

    pub fn tick(&mut self, time_in_ms: f32) {
        let deltaT = time_in_ms / 1000.0;
        let mut forces_to_apply: Vec<(Vec2d, Vec2d)> = Vec::new();
        self.effects
            .for_each(|effect: &mut Effect, _: usize| match effect {
                Effect::Circular(e) => {
                    e.time_to_live -= deltaT;
                    if e.time_to_live > 0.0 {
                        e.radius += e.expansion_speed * deltaT;

                        // all vertices within the radius of the effect
                        // are affected
                        let start_pos = e.center - (Vec2d::new(e.radius, e.radius) * 0.5f32);
                        let num_x_steps = (e.radius / GRID_DISTANCE) as usize;
                        let num_y_steps = (e.radius / GRID_DISTANCE) as usize;

                        for i in 0..=num_y_steps {
                            for j in 0..=num_x_steps {
                                let current_pos = start_pos
                                    + Vec2d::new(
                                        j as f32 * GRID_DISTANCE,
                                        i as f32 * GRID_DISTANCE,
                                    );

                                let mut the_force = e.center - current_pos;
                                if the_force.is_not_zero() {
                                    // force gets weaker as it closes in on its ttl
                                    the_force = the_force.normalized() * e.time_to_live * 500.0;
                                } else {
                                    the_force = Vec2d::default();
                                }

                                forces_to_apply.push((the_force, current_pos));
                            }
                        }
                    }
                }
            });

        self.effects.garbage_collect_filter(|x| match x {
            Effect::Circular(x) => x.time_to_live <= 0.0,
        });

        for (force, pos) in forces_to_apply {
            self.apply_force(force, pos, deltaT);
        }

        for elem in self.grid.iter_mut() {
            elem.mov();
            elem.set_dir_back();
        }
    }

    pub fn render(
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
                    .transform(&self.grid[i + (current_row * row_count)].position());
                let p2: Vec2d = screen_space_transform
                    .transform(&self.grid[i + 1 + (current_row * row_count)].position());
                let _ = draw::draw_line(canvas, &p1, &p2, Color::BLUE);
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
                    .transform(&self.grid[i * row_count + current_col].position());
                let p2: Vec2d = screen_space_transform
                    .transform(&self.grid[(i + 1) * row_count + current_col].position());
                let _ = draw::draw_line(canvas, &p1, &p2, Color::BLUE);
                i += 1;
            }
            current_col += 1;
        }
    }
}
