use sdl2::{pixels::Color, rect::Point};

use crate::{draw, vecmath::Vec2d};

pub const StarShip: [Vec2d; 4] = [
    Vec2d::new(0.0, 0.0),
    Vec2d::new(1.0, -1.0),
    Vec2d::new(0.0, 2.0),
    Vec2d::new(-1.0, -1.0),
];

pub const LanderScale: Vec2d = Vec2d::new(15.0, 15.0);

pub const FlameA: [Vec2d; 3] = [
    Vec2d::new(-0.5, -1.5),
    Vec2d::new(0.5, -1.5),
    Vec2d::new(0.0, -3.0),
];

pub const FlameB: [Vec2d; 3] = [
    Vec2d::new(-0.5, -1.5),
    Vec2d::new(0.5, -1.5),
    Vec2d::new(0.0, -1.5),
];

pub fn render_game_over(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    window_center: Vec2d,
) {
    draw::draw_text_centered(
        canvas,
        "GAME OVER",
        60,
        Point::new(window_center.x as i32, window_center.y as i32),
        Color::RGB(255, 0, 0),
    )
    .unwrap();
}

pub fn render_won_text(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    window_center: Vec2d,
) {
    draw::draw_text_centered(
        canvas,
        "WON",
        60,
        Point::new(window_center.x as i32, window_center.y as i32),
        Color::RGB(255, 0, 0),
    )
    .unwrap()
}
