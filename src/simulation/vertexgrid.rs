use sdl2::pixels::Color;

use crate::{
    draw,
    vecmath::{TransformationMatrix, Vec2d},
};

use super::{vertex::Vertex, GRID_DISTANCE, WORLD_SIZE};

pub struct VertexGrid {
    grid: Vec<Vertex>,
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
        Self { grid }
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
        if (index > num_coll) {
            indices.push(index - num_coll);
        }
        if (index + num_coll < self.grid.len()) {
            indices.push(index + num_coll);
        }
        if (index % num_coll != 0) {
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

    pub fn tick(&mut self) {
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
