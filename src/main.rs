use sdl2::event::{Event, WindowEvent};
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::Texture;
use sdl2::sys::SDL_GetTicks;
use std::collections::HashMap;

use simulation::DirectionKey;

mod collision;
mod draw;
mod graphics;
mod hud;
mod simulation;
mod sound;
mod vecmath;

pub const WINDOW_WIDTH: u32 = 1024;
pub const WINDOW_HEIGHT: u32 = 768;

fn new_simultaion() -> simulation::World {
    return simulation::World::new(WINDOW_WIDTH, WINDOW_HEIGHT);
}

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", WINDOW_WIDTH, WINDOW_HEIGHT)
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

    let mut sim = new_simultaion();
    let texture_c = canvas.texture_creator();
    let mut star = texture_c.load_texture("./assets/star.png").unwrap();
    star.set_blend_mode(sdl2::render::BlendMode::Add);

    let mut neon = texture_c.load_texture("./assets/neon.png").unwrap();
    neon.set_blend_mode(sdl2::render::BlendMode::Add);

    let mut halo = texture_c.load_texture("./assets/halo.png").unwrap();
    halo.set_blend_mode(sdl2::render::BlendMode::Add);

    let mut texture_dict: HashMap<String, Texture> = HashMap::new();
    texture_dict.insert("star".to_string(), star);
    texture_dict.insert("neon".to_string(), neon);
    texture_dict.insert("halo".to_string(), halo);

    let mut loop_time = unsafe { SDL_GetTicks() };

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
                        match keycode {
                            Some(keycode) => match keycode {
                                Keycode::W => sim.shoot(DirectionKey::Up, true),
                                Keycode::S => sim.shoot(DirectionKey::Down, true),
                                Keycode::A => sim.shoot(DirectionKey::Left, true),
                                Keycode::D => sim.shoot(DirectionKey::Right, true),
                                Keycode::Up => sim.direction_toggle(DirectionKey::Up, true),
                                Keycode::Down => sim.direction_toggle(DirectionKey::Down, true),
                                Keycode::Left => sim.direction_toggle(DirectionKey::Left, true),
                                Keycode::Right => sim.direction_toggle(DirectionKey::Right, true),
                                Keycode::M => sim.toggle_background_music(),
                                Keycode::R => sim = new_simultaion(),
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
                        match keycode {
                            Some(keycode) => match keycode {
                                Keycode::W => sim.shoot(DirectionKey::Up, false),
                                Keycode::S => sim.shoot(DirectionKey::Down, false),
                                Keycode::A => sim.shoot(DirectionKey::Left, false),
                                Keycode::D => sim.shoot(DirectionKey::Right, false),
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

        // The rest of the game loop goes here...
        let time_taken = unsafe { SDL_GetTicks() } - loop_time;
        println!("taken {}", time_taken);
        sim.tick(time_taken as f32, 2.0);
        loop_time = unsafe { SDL_GetTicks() };
    }

    Ok(())
}
