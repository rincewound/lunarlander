use crate::vecmath::Vec2d;

pub struct Vertex {
    main_position: Vec2d,
    position: Vec2d,
    direction: Vec2d,
}

impl Vertex {
    pub fn new(pos: Vec2d) -> Self {
        Self {
            position: pos,
            main_position: pos,
            direction: Vec2d::new(0.0, 0.0),
        }
    }

    pub fn mov(&mut self) {
        self.position = self.position + self.direction;
        self.direction = self.direction * 0.5;
    }

    pub fn add_to_dir(&mut self, dir: Vec2d) {
        self.direction = self.direction + dir;
        if self.direction.len() > 10.0 {
            self.direction = self.direction.normalized() * 10.0;
        }
    }

    pub fn set_dir_back(&mut self) {
        let dir = self.main_position - self.position;
        if dir.len() > 0.01 {
            let length = dir.len();
            let mut mult = length / 5.0;
            if mult > 1.0 {
                mult = 1.0;
            } else if mult < 0.0 {
                mult = 0.0;
            }
            self.add_to_dir(dir.normalized() * mult * 5.0);
        }
    }

    pub fn position(&self) -> Vec2d {
        self.position
    }
}
