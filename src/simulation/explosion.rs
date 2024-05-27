use rand::Rng;

use crate::vecmath::Vec2d;

#[derive(Clone)]
pub struct Explosion {
    pub position: Vec2d,
    pub frame_count: u32,
    pub sparc_dir: Vec<Vec2d>,
}

impl Explosion {
    pub fn new(position: Vec2d) -> Self {
        let mut rnd = rand::thread_rng();
        let count_sparcs = rnd.gen_range(50..100);
        let mut init_sparc = Vec::new();
        for _ in 0..count_sparcs {
            let new_sparc = Vec2d::from_angle(rnd.gen_range(0.0..360.0));
            let new_sparc = new_sparc * rnd.gen_range(0.5..2.0);
            init_sparc.push(new_sparc);
        }
        Self {
            position: position,
            frame_count: 0,
            sparc_dir: init_sparc,
        }
    }
}
