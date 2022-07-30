use crate::vecmath::Vec2d;

const LanderTop: [Vec2d; 4] = [
    Vec2d::new(-2.0, 3.0),
    Vec2d::new(-1.0, 4.0),
    Vec2d::new(1.0, 4.0),
    Vec2d::new(2.0, 3.0),
];

const LanderMiddle: [Vec2d; 4] = [
    Vec2d::new(-2.0, 3.0),
    Vec2d::new(2.0, 3.0),
    Vec2d::new(3.0, 0.0),
    Vec2d::new(-3.0, 0.0),
];

const LanderBottom: [Vec2d; 4] = [
    Vec2d::new(-2.0, 0.0),
    Vec2d::new(2.0, 0.0),
    Vec2d::new(2.0, -2.0),
    Vec2d::new(-2.0, -2.0),
];

const LanderDrive: [Vec2d; 4] = [
    Vec2d::new(-1.0, -2.0),
    Vec2d::new(1.0, -2.0),
    Vec2d::new(-2.0, -3.0),
    Vec2d::new(2.0, -3.0),
];

const LeftLeg: [Vec2d; 2] = [Vec2d::new(-2.0, 0.0), Vec2d::new(-3.0, -3.0)];

const RightLeg: [Vec2d; 2] = [Vec2d::new(2.0, 0.0), Vec2d::new(3.0, -3.0)];
