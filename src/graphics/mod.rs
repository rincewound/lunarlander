use crate::vecmath::Vec2d;

pub const LanderTop: [Vec2d; 4] = [
    Vec2d::new(-2.0, 3.0),
    Vec2d::new(-1.0, 4.0),
    Vec2d::new(1.0, 4.0),
    Vec2d::new(2.0, 3.0),
];

pub const LanderMiddle: [Vec2d; 4] = [
    Vec2d::new(-2.0, 3.0),
    Vec2d::new(2.0, 3.0),
    Vec2d::new(3.0, 0.0),
    Vec2d::new(-3.0, 0.0),
];

pub const LanderBottom: [Vec2d; 4] = [
    Vec2d::new(-2.0, 0.0),
    Vec2d::new(2.0, 0.0),
    Vec2d::new(2.0, -2.0),
    Vec2d::new(-2.0, -2.0),
];

pub const LanderDrive: [Vec2d; 4] = [
    Vec2d::new(-1.0, -2.0),
    Vec2d::new(1.0, -2.0),
    Vec2d::new(-2.0, -3.0),
    Vec2d::new(2.0, -3.0),
];

pub const LanderScale: Vec2d = Vec2d::new(5.0, 5.0);
pub const LanderBoundBoxOrigin: Vec2d = Vec2d::new(-3.0, 4.0);
pub const LanderWidth: u32 = 6;
pub const LanderHeight: u32 = 7;


pub const LeftLeg: [Vec2d; 2] = [Vec2d::new(-2.0, 0.0), Vec2d::new(-3.0, -3.0)];

pub const RightLeg: [Vec2d; 2] = [Vec2d::new(2.0, 0.0), Vec2d::new(3.0, -3.0)];

pub const FlameA: [Vec2d; 3] = [
    Vec2d::new(-1.0, -3.0),
    Vec2d::new(1.0, -3.0),
    Vec2d::new(0.0, -6.0),
];

pub const FlameB: [Vec2d; 3] = [
    Vec2d::new(-1.0, -3.0),
    Vec2d::new(1.0, -3.0),
    Vec2d::new(0.0, -4.0),
];
