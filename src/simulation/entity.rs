use crate::vecmath::{self, TransformationMatrix, Vec2d};

use super::{objectstore::ObjectDefault, BorderBehavior, WORLD_SIZE};

#[derive(Clone, Debug, PartialEq)]
pub struct Entity {
    position: Vec2d,
    angle: f32,          // angle in rad
    direction: Vec2d,    // non normalized, has speed integrated!
    acceleration: Vec2d, // non normalized, has force integrated!
    max_velocity: f32,
    border_behavior: BorderBehavior,
    update: bool,
}

impl ObjectDefault for Entity {
    fn default() -> Self {
        Entity::default()
    }
}

impl Entity {
    pub(crate) fn default() -> Self {
        Entity {
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

    pub fn velocity(&self) -> f32 {
        return self.direction.len();
    }

    pub fn acceleration(&self) -> Vec2d {
        self.acceleration
    }

    pub fn direction(&self) -> Vec2d {
        self.direction
    }

    pub fn position(&self) -> Vec2d {
        self.position
    }

    pub fn angle(&self) -> f32 {
        self.angle
    }

    pub fn set_angle(&mut self, angle: f32) {
        self.angle = angle;
    }

    pub fn update(&self) -> bool {
        self.update
    }

    pub fn set_direction(&mut self, direction: Vec2d) {
        self.direction = direction;
    }

    pub fn max_velocity(&self) -> f32 {
        self.max_velocity
    }

    pub fn border_behavior(&self) -> &BorderBehavior {
        &self.border_behavior
    }

    pub fn set_max_velocity(&mut self, max_velocity: f32) {
        self.max_velocity = max_velocity;
    }

    pub fn set_border_behavior(&mut self, border_behavior: BorderBehavior) {
        self.border_behavior = border_behavior;
    }

    pub fn physics_tick(&mut self, sim_time_in_seconds: f32, num_ticks: usize) {
        for _ in 0..num_ticks {
            if !self.update() {
                return;
            }
            // update direction by applying acceleration:
            let accel_fragment = self.acceleration().clone() * (sim_time_in_seconds);
            self.set_direction(self.direction() + accel_fragment);

            self.set_direction(if self.direction().len() > self.max_velocity() {
                self.direction().normalized() * self.max_velocity()
            } else {
                self.direction()
            });
            let mut new_pos = self.position() + self.direction().clone() * (sim_time_in_seconds);

            if new_pos.x < 0.0
                || new_pos.y < 0.0
                || new_pos.x > WORLD_SIZE.x
                || new_pos.y > WORLD_SIZE.y
            {
                match self.border_behavior() {
                    BorderBehavior::Dismiss => {
                        // TODO: destroy missile/entity
                    }
                    BorderBehavior::Bounce => {
                        self.set_direction(self.direction() * -1.0);
                        new_pos =
                            self.position() + self.direction().clone() * (sim_time_in_seconds);
                    }
                    BorderBehavior::BounceSlowdown => {
                        self.set_direction(self.direction() * -0.2);
                        new_pos =
                            self.position() + self.direction().clone() * (sim_time_in_seconds);
                    }
                }
            }
            self.set_position(new_pos);
        }
    }
}
