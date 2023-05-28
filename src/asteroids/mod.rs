use rand::Rng;
use std::f32::consts::PI;

use crate::{
    simulation::Entity,
    vecmath::{self, Vec2d},
};

#[derive(Debug, PartialEq, Clone)]
pub struct Asteroid {
    pub entity_id: usize,
    scale: usize,
    border_points: Vec<Vec2d>,
}

impl Asteroid {
    pub fn new(entity_id: usize, scale: usize) -> Self {
        Asteroid {
            entity_id: entity_id,
            scale: scale,
            border_points: Asteroid::new_uniform(),
        }
    }

    fn new_uniform() -> Vec<Vec2d> {
        let mut rng = rand::thread_rng();
        let mut circle_points: Vec<Vec2d> = Vec::new();
        let max_border_points = rng.gen_range(10..18);
        let angle_step_rad = 2.0 * PI / max_border_points as f32;
        for step_idx in 0..max_border_points {
            let (x, y) = (step_idx as f32 * angle_step_rad).sin_cos();
            let rand_fact = rng.gen_range(0.8..1.2);
            circle_points.push(Vec2d {
                x: x * rand_fact,
                y: y * rand_fact,
            })
        }
        circle_points
    }

    pub fn split(&self, entity_ids: Vec<usize>) -> Vec<Self> {
        if self.scale > 1 {
            let new_scale = self.scale - 1;
            entity_ids
                .iter()
                .map(|&id| Asteroid {
                    entity_id: id,
                    scale: new_scale,
                    border_points: Asteroid::new_uniform(),
                })
                .collect()
        }
        else
        {
            Vec::new()
        }
    }

    pub fn get_transformed_hull(&self, entity: &Entity) -> Vec<Vec2d> {
        assert_eq!(self.entity_id, entity.get_id());
        let entity_transform = entity.get_transform();
        let scale_factor = 7.0 + (self.scale as f32 * 7.0);
        let scale_transform = vecmath::TransformationMatrix::scale(scale_factor, scale_factor);
        let transform = entity_transform * scale_transform;
        let hull_points = transform.transform_many(&self.border_points);
        return hull_points;
    }
}

#[cfg(test)]
mod tests {
    use crate::asteroids::*;

    #[test]
    fn test_asteroid_new_gen() {
        let ast = Asteroid::new(42, 5);
        println!("border point list: {:?}", ast.border_points);
        assert_eq!(ast.entity_id, 42);
        assert_eq!(ast.border_points.len(), 14)
    }
}
