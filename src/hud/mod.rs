use crate::draw;
use crate::vecmath::Vec2d;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;

pub struct Hud {
    position: Vec2d,
    direction: Vec2d,
    deaths: u32,
}

impl Hud {
    pub fn new() -> Self {
        Self {
            position: Vec2d::new(0.0, 0.0),
            direction: Vec2d::new(0.0, 0.0),
            deaths: 0,
        }
    }

    pub fn from(position: Vec2d, direction: Vec2d, deaths: u32) -> Self {
        Self {
            position,
            direction,
            deaths,
        }
    }

    pub fn update(&mut self, position: Vec2d, direction: Vec2d, deaths: u32) {
        self.position = position;
        self.direction = direction;
        self.deaths = deaths;
    }

    pub fn updatePosition(&mut self, position: Vec2d) {
        self.position = position;
    }

    pub fn updateDirection(&mut self, direction: Vec2d) {
        self.direction = direction;
    }

    pub fn updateDeaths(&mut self, deaths: u32) {
        self.deaths = deaths;
    }

    pub fn render(&self, canvas: &mut Canvas<Window>) {
        let hud_position = format!("Position: x = {}, y = {}", self.position.x, self.position.y);
        let hud_direction = format!(
            "Direction: x = {}, y = {}",
            self.direction.x, self.direction.y
        );
        let hud_deaths = format!("Deaths: {}", self.deaths);

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
            &hud_deaths,
            10,
            Point::new(0, 20),
            Color::RGB(0, 255, 0),
        )
        .unwrap();
    }
}
