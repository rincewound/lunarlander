use rand::Rng;
use std::f32::consts::PI;

use crate::{vecmath::Vec2d, simulation::Entity};

#[derive(Debug)]
pub struct Asteroid {
    pub entity_id: usize,
    pub border_points: Vec<Vec2d>,
}

impl Asteroid {
    pub fn new(entity_id: usize) -> Self {
        Asteroid {
            entity_id: entity_id,
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

    pub fn get_transformed_hull(&self, entity: &Entity) -> Vec<Vec2d>
    {
        return self.border_points.clone();
    } 
}

#[cfg(test)]
mod tests {
    use std::borrow::Borrow;

    use crate::asteroids::*;

    #[test]
    fn test_asteroid_new_gen() {
        let ast = Asteroid::new(42);
        println!("border point list: {:?}", ast.border_points);
        assert_eq!(ast.entity_id, 42);
        assert_eq!(ast.border_points.len(), 14)
    }
}
