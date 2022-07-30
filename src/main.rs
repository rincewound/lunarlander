use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use std::time::Duration;

mod draw;
mod map;
mod simulation;
mod vecmath;

pub fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("rust-sdl2 demo: Video", 800, 600)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window.into_canvas().build().map_err(|e| e.to_string())?;

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,
                _ => {}
            }
        }

        canvas.clear();

        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        draw_example(&mut canvas);

        canvas.present();
        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        // The rest of the game loop goes here...
    }

    Ok(())
}

fn draw_example(canvas: &mut Canvas<Window>) 
{
    let p1 = Box::new(vecmath::Vec2d::new(0.0, 0.0));
    let p2 = Box::new(vecmath::Vec2d::new(100.0, 100.0));
    let p3 = Box::new(vecmath::Vec2d::new(200.0, 500.0));

    let mut points = Vec::new();
    points.push(p1);
    points.push(p2);
    points.push(p3);
    draw::draw_lines(canvas, &points, Color::RGB(255, 255, 255)).unwrap();

    let origin = vecmath::Vec2d::new(300.0, 300.0);
    draw::draw_rect(canvas, &origin, 50, 100, Color::RGB(0, 0, 255), true).unwrap();
}
