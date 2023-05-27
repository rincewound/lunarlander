use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture};
use sdl2::video::Window;
use vecmath::Vec2d;
use std::collections::HashMap;
use std::time::Duration;
use sdl2::image::LoadTexture;

mod asteroids;
mod collision;
mod draw;
mod graphics;
mod hud;
mod map;
mod simulation;
mod vecmath;

pub const window_width: u32 = 800;
pub const window_height: u32 = 600;
pub const window_center: Vec2d = Vec2d::new(window_width as f32 / 2.0, window_height as f32 / 2.0);

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", window_width, window_height)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    let mut sim = simulation::World::new(window_width, window_height);

    let texture_c = canvas.texture_creator();
    let mut star = texture_c
        .load_texture("./assets/star.png")
        .unwrap();
    star.set_blend_mode(sdl2::render::BlendMode::Add);

    let mut texture_dict: HashMap<String, Texture> = HashMap::new();
    texture_dict.insert("star".to_string(), star);

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    sim.shoot();
                }
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    sim.thrust_toggle(true);
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    sim.thrust_toggle(false);
                }

                Event::KeyUp {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    sim.rotation_left_toggle(false);
                }

                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    sim.rotation_left_toggle(true);
                }

                Event::KeyUp {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    sim.rotation_right_toggle(false);
                }

                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    sim.rotation_right_toggle(true);
                }
                _ => {}
            }
        }

        canvas.clear();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        sim.render(&mut canvas, &texture_dict);

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        // The rest of the game loop goes here...
        sim.tick(50.0, 10.0);
    }

    Ok(())
}
