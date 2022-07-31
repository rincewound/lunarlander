use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::time::Duration;

mod draw;
mod hud;
mod graphics;
mod map;
mod simulation;
mod vecmath;
mod collision;

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window_width = 800;
    let window_height = 600;

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
                    sim.thrust_toggle(true);
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Space),
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
        sim.render(&mut canvas);

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        // The rest of the game loop goes here...
        sim.tick(50.0, 10.0);
    }

    Ok(())
}