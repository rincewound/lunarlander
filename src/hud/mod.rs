use crate::draw;
use crate::vecmath::Vec2d;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;

pub struct Hud {
    position: Vec2d,
    direction: Vec2d,
    angle: f32,
    score: u32,
    asteroids: u32,
}

impl Hud {
    pub fn new() -> Self {
        Self {
            position: Vec2d::new(0.0, 0.0),
            direction: Vec2d::new(0.0, 0.0),
            angle: 0.0,
            score: 0,
            asteroids: 0,
        }
    }

    pub fn from(position: Vec2d, direction: Vec2d, angle: f32, score: u32, asteroids: u32) -> Self {
        Self {
            position,
            direction,
            angle,
            score,
            asteroids,
        }
    }

    pub fn update(
        &mut self,
        position: Vec2d,
        direction: Vec2d,
        angle: f32,
        score: u32,
        asteroids: u32,
    ) {
        self.position = position;
        self.direction = direction;
        self.angle = angle;
        self.score = score;
        self.asteroids = asteroids;
    }

    pub fn updatePosition(&mut self, position: Vec2d) {
        self.position = position;
    }

    pub fn updateDirection(&mut self, direction: Vec2d) {
        self.direction = direction;
    }

    pub fn updateAngle(&mut self, angle: f32) {
        self.angle = angle;
    }

    pub fn updateScore(&mut self, score: u32) {
        self.score = score;
    }

    pub fn render(&self, canvas: &mut Canvas<Window>) {
        let hud_position = format!("Position: x = {}, y = {}", self.position.x, self.position.y);
        let hud_direction = format!(
            "Direction: x = {}, y = {}",
            self.direction.x, self.direction.y
        );
        let hud_angle = format!("Angle: {}", self.angle);
        let hud_score = format!("Score: {}", self.score);
        let hud_asteroids = format!("Asteroids: {}", self.asteroids);

        draw::draw_text(
            canvas,
            &hud_position,
            10,
            Point::new(0, 0),
            Color::RGB(0, 255, 0),
        )
        .unwrap();
        draw::draw_text(
            canvas,
            &hud_direction,
            10,
            Point::new(0, 10),
            Color::RGB(0, 255, 0),
        )
        .unwrap();
        draw::draw_text(
            canvas,
            &hud_angle,
            10,
            Point::new(0, 20),
            Color::RGB(0, 255, 0),
        )
        .unwrap();
        draw::draw_text(
            canvas,
            &hud_score,
            10,
            Point::new(0, 30),
            Color::RGB(0, 255, 0),
        )
        .unwrap();
        draw::draw_text(
            canvas,
            &hud_asteroids,
            10,
            Point::new(0, 40),
            Color::RGB(0, 255, 0),
        )
        .unwrap();
    }
}
