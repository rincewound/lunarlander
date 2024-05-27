use sdl2::controller::{Axis, GameController};
use sdl2::event::{Event, WindowEvent};
use sdl2::image::LoadTexture;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::{Canvas, Texture};
use sdl2::sys::SDL_GetTicks;
use sdl2::video::Window;
use std::collections::HashMap;

use simulation::{
    World, BIT_DOWN, BIT_LEFT, BIT_RIGHT, BIT_SHOOT_DOWN, BIT_SHOOT_LEFT, BIT_SHOOT_RIGHT,
    BIT_SHOOT_UP, BIT_UP,
};

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
    let mut w = WINDOW_WIDTH;
    let mut h = WINDOW_HEIGHT;

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let gamepad = sdl_context.game_controller()?;

    let _pad: Option<GameController> = if gamepad.is_game_controller(0) {
        println!("Using {}", gamepad.name_for_index(0).unwrap());
        Some(gamepad.open(0).unwrap())
    } else {
        None
    };

    let window = video_subsystem
        .window("GWARS", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .resizable()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas: Canvas<Window> = window.into_canvas().build().map_err(|e| e.to_string())?;

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
                                Keycode::Up => sim.modify_control_bit(BIT_SHOOT_UP, true),
                                Keycode::Down => sim.modify_control_bit(BIT_SHOOT_DOWN, true),
                                Keycode::Left => sim.modify_control_bit(BIT_SHOOT_LEFT, true),
                                Keycode::Right => sim.modify_control_bit(BIT_SHOOT_RIGHT, true),
                                Keycode::W => sim.modify_control_bit(BIT_UP, true),
                                Keycode::S => sim.modify_control_bit(BIT_DOWN, true),
                                Keycode::A => sim.modify_control_bit(BIT_LEFT, true),
                                Keycode::D => sim.modify_control_bit(BIT_RIGHT, true),
                                Keycode::M => sim.toggle_background_music(),
                                Keycode::R => restart(&mut sim, w, h),
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
                                Keycode::Up => sim.modify_control_bit(BIT_SHOOT_UP, false),
                                Keycode::Down => sim.modify_control_bit(BIT_SHOOT_DOWN, false),
                                Keycode::Left => sim.modify_control_bit(BIT_SHOOT_LEFT, false),
                                Keycode::Right => sim.modify_control_bit(BIT_SHOOT_RIGHT, false),
                                Keycode::W => sim.modify_control_bit(BIT_UP, false),
                                Keycode::S => sim.modify_control_bit(BIT_DOWN, false),
                                Keycode::A => sim.modify_control_bit(BIT_LEFT, false),
                                Keycode::D => sim.modify_control_bit(BIT_RIGHT, false),
                                _ => continue,
                            },
                            None => continue,
                        }
                    }
                }
                Event::Window { win_event, .. } => match win_event {
                    WindowEvent::SizeChanged(width, height) => {
                        w = width as u32;
                        h = height as u32;
                        sim.update_window_size(width as f32, height as f32)
                    }
                    _ => continue,
                },
                Event::ControllerAxisMotion {
                    timestamp: _,
                    which: _,
                    axis,
                    value,
                } => {
                    println!("Axis: {:?} value: {:?}", axis, value);
                    match axis {
                        Axis::LeftX => {
                            sim.modify_control_bit(BIT_LEFT, value < -2000);
                            sim.modify_control_bit(BIT_RIGHT, value > 2000);
                        }
                        Axis::LeftY => {
                            sim.modify_control_bit(BIT_UP, value < -2000);
                            sim.modify_control_bit(BIT_DOWN, value > 2000);
                        }
                        Axis::RightX => {
                            sim.modify_control_bit(BIT_SHOOT_LEFT, value < -2000);
                            sim.modify_control_bit(BIT_SHOOT_RIGHT, value > 2000);
                        }
                        Axis::RightY => {
                            sim.modify_control_bit(BIT_SHOOT_UP, value < -2000);
                            sim.modify_control_bit(BIT_SHOOT_DOWN, value > 2000);
                        }
                        Axis::TriggerLeft => restart(&mut sim, w, h),
                        Axis::TriggerRight => todo!(),
                    }
                }
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
        //println!("taken {}", time_taken);
        sim.tick(time_taken as f32, 2.0);
        loop_time = unsafe { SDL_GetTicks() };
    }

    Ok(())
}

fn restart(sim: &mut World, w: u32, h: u32) {
    *sim = new_simultaion();
    sim.update_window_size(w as f32, h as f32);
}
