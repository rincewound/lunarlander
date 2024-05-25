use sdl2::{pixels::Color, rect::Point};

use crate::{draw, vecmath::Vec2d};

pub const STARSHIP: [Vec2d; 4] = [
    Vec2d::new(0.0, 0.0),
    Vec2d::new(1.0, -1.0),
    Vec2d::new(0.0, 2.0),
    Vec2d::new(-1.0, -1.0),
];

pub const ENTITY_SCALE: Vec2d = Vec2d::new(15.0, 15.0);
pub const STARSHIP_COLOR: Color = Color::RGBA(255, 255, 188, 255);

pub const RECT_ENEMY: [Vec2d; 4] = [
    Vec2d::new(1.0, 1.0),
    Vec2d::new(-1.0, 1.0),
    Vec2d::new(-1.0, -1.0),
    Vec2d::new(1.0, -1.0),
];

pub const RECT_ENEMY_COLOR: Color = Color::RGBA(255, 0, 0, 255);

pub const ROMBUS_ENEMY: [Vec2d; 4] = [
    Vec2d::new(2.0, 0.0),
    Vec2d::new(0.0, 1.0),
    Vec2d::new(-2.0, 0.0),
    Vec2d::new(0.0, -1.0),
];

pub const MISSILE: [Vec2d; 5] = [
    Vec2d::new(0.0, 0.0),
    Vec2d::new(-0.5, 0.5),
    Vec2d::new(-2.0, 0.5),
    Vec2d::new(-2.0, -0.5),
    Vec2d::new(-0.5, -0.5),
];

pub const ROMBUS_ENEMY_COLOR: Color = Color::RGBA(255, 255, 0, 255);

pub const FLAME_A: [Vec2d; 3] = [
    Vec2d::new(-0.5, -1.5),
    Vec2d::new(0.5, -1.5),
    Vec2d::new(0.0, -3.0),
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
