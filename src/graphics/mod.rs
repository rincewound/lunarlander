use sdl2::{pixels::Color, rect::Point};

use crate::{draw, vecmath::Vec2d, window_center};



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

pub const BBox: [Vec2d; 4] = [
    Vec2d::new(-3.0, 4.0),
    Vec2d::new(3.0, 4.0),
    Vec2d::new(3.0, -3.0),
    Vec2d::new(-3.0, -3.0),
];


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

pub fn renderGameOver(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
    draw::draw_text_centered(
        canvas,
        "GAME OVER",
        60,
        Point::new( window_center.x as i32, window_center.y as i32),
        Color::RGB(255, 0, 0),
    )
    .unwrap();
}

pub fn renderWonText(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
    draw::draw_text_centered(
        canvas,
        "WON",
        60,
        Point::new(window_center.x as i32, window_center.y as i32),
        Color::RGB(255, 0, 0),
    )
    .unwrap()
}
