use sdl2::event::{Event, WindowEvent};
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::Texture;
use std::collections::HashMap;
use std::time::Duration;

use simulation::DirectionKey;

mod asteroids;
mod collision;
mod draw;
mod graphics;
mod hud;
mod simulation;
mod sound;
mod vecmath;

pub const window_width: u32 = 800;
pub const window_height: u32 = 600;

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", window_width, window_height)
        .position_centered()
        .resizable()
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
    let mut star = texture_c.load_texture("./assets/star.png").unwrap();
    star.set_blend_mode(sdl2::render::BlendMode::Add);

    let mut neon = texture_c.load_texture("./assets/neon.png").unwrap();
    neon.set_blend_mode(sdl2::render::BlendMode::Add);

    let mut texture_dict: HashMap<String, Texture> = HashMap::new();
    texture_dict.insert("star".to_string(), star);
    texture_dict.insert("neon".to_string(), neon);

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                Event::KeyDown {
                    keycode, repeat, ..
                } => {
                    if !repeat {
                        println!("event: {:?}", { event });
                        match keycode {
                            Some(keycode) => match keycode {
                                Keycode::Up => sim.direction_toggle(DirectionKey::Up, true),
                                Keycode::Down => sim.direction_toggle(DirectionKey::Down, true),
                                Keycode::Left => sim.direction_toggle(DirectionKey::Left, true),
                                Keycode::Right => sim.direction_toggle(DirectionKey::Right, true),
                                Keycode::Space => sim.shoot(),
                                Keycode::S => sim.toggle_background_music(),
                                _ => continue,
                            },
                            None => continue,
                        }
                    }
                }
                Event::KeyUp {
                    keycode, repeat, ..
                } => {
                    if !repeat {
                        println!("event: {:?}", { event });
                        match keycode {
                            Some(keycode) => match keycode {
                                Keycode::Up => sim.direction_toggle(DirectionKey::Up, false),
                                Keycode::Down => sim.direction_toggle(DirectionKey::Down, false),
                                Keycode::Left => sim.direction_toggle(DirectionKey::Left, false),
                                Keycode::Right => sim.direction_toggle(DirectionKey::Right, false),
                                _ => continue,
                            },
                            None => continue,
                        }
                    }
                }
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::SizeChanged(width, height) => {
                        sim.update_window_size(width as f32, height as f32)
                    }
                    _ => continue,
                },
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
