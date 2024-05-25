use crate::vecmath;
use sdl2::pixels::Color;
pub use sdl2::rect::Point;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf;
use sdl2::video::Window;
use std::path::Path;

use crate::vecmath::{TransformationMatrix, Vec2d};

pub fn neon_draw_line(
    canvas: &mut Canvas<Window>,
    from: &Vec2d,
    to: &Vec2d,
    color: Color,
) -> Result<(), String> {
    canvas.set_draw_color(color);

    // We calculate a rectangle that is wider than the line and will be
    // filled with a neon texture:
    const width: f32 = 2.0f32;
    let line_dir = (to.clone() - from.clone()).normalized();
    let rotPlus90 = TransformationMatrix::rotate(std::f32::consts::FRAC_PI_2);
    let rotMinus90 = TransformationMatrix::rotate(-std::f32::consts::FRAC_PI_2);

    let toUpperPoint = rotPlus90.transform(&line_dir) * width;
    let toLowerPoint = rotMinus90.transform(&line_dir) * width;

    let p0 = from.clone() + toUpperPoint;
    let p1 = to.clone() + toUpperPoint;
    let p2 = to.clone() + toLowerPoint;
    let p3 = from.clone() + toLowerPoint;

    let _ = canvas.draw_line(
        Point::new(p0.x as i32, p0.y as i32),
        Point::new(p1.x as i32, p1.y as i32),
    );

    let _ = canvas.draw_line(
        Point::new(p1.x as i32, p1.y as i32),
        Point::new(p2.x as i32, p2.y as i32),
    );
    let _ = canvas.draw_line(
        Point::new(p2.x as i32, p2.y as i32),
        Point::new(p3.x as i32, p3.y as i32),
    );

    let _ = canvas.draw_line(
        Point::new(p3.x as i32, p3.y as i32),
        Point::new(p0.x as i32, p0.y as i32),
    );

    return canvas.draw_line(
        Point::new(from.x as i32, from.y as i32),
        Point::new(to.x as i32, to.y as i32),
    );
}

pub fn draw_line(
    canvas: &mut Canvas<Window>,
    from: &Vec2d,
    to: &Vec2d,
    color: Color,
) -> Result<(), String> {
    canvas.set_draw_color(color);
    return canvas.draw_line(
        Point::new(from.x as i32, from.y as i32),
        Point::new(to.x as i32, to.y as i32),
    );
}

pub fn draw_lines(
    canvas: &mut Canvas<Window>,
    points: &Vec<Vec2d>,
    color: Color,
    close: bool,
) -> Result<(), String> {
    canvas.set_draw_color(color);

    for idx in 1..points.len() {
        if let Err(error) = draw_line(canvas, &points[idx - 1], &points[idx], color) {
            return Err(error);
        }
    }

    if close {
        if let Err(error) = draw_line(canvas, &points[points.len() - 1], &points[0], color) {
            return Err(error);
        }
    }

    Ok(())
}

pub fn neon_draw_lines(
    canvas: &mut Canvas<Window>,
    points: &Vec<Vec2d>,
    color: Color,
    close: bool,
) -> Result<(), String> {
    canvas.set_draw_color(color);

    for idx in 1..points.len() {
        if let Err(error) = neon_draw_line(canvas, &points[idx - 1], &points[idx], color) {
            return Err(error);
        }
    }

    if close {
        if let Err(error) = neon_draw_line(canvas, &points[points.len() - 1], &points[0], color) {
            return Err(error);
        }
    }

    Ok(())
}

pub fn draw_rect<T>(
    canvas: &mut Canvas<Window>,
    origin: &Vec2d,
    width: u32,
    height: u32,
    color: Color,
    fill: bool,
) -> Result<(), String> {
    canvas.set_draw_color(color);
    let rect = Rect::new(origin.x as i32, origin.y as i32, width, height);

    if fill {
        return canvas.fill_rect(rect);
    } else {
        return canvas.draw_rect(rect);
    }
}

pub fn draw_text(
    canvas: &mut Canvas<Window>,
    text: &str,
    font_size: u16,
    origin: Point,
    color: Color,
) -> Result<(), String> {
    draw_text_raw(canvas, text, font_size, origin, color, false)
}

pub fn draw_text_centered(
    canvas: &mut Canvas<Window>,
    text: &str,
    font_size: u16,
    origin: Point,
    color: Color,
) -> Result<(), String> {
    draw_text_raw(canvas, text, font_size, origin, color, true)
}
fn draw_text_raw(
    canvas: &mut Canvas<Window>,
    text: &str,
    font_size: u16,
    origin: Point,
    color: Color,
    centered: bool,
) -> Result<(), String> {
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let font_path = Path::new("assets/DejaVuSansMono.ttf");
    let font = ttf_context.load_font(font_path, font_size)?;

    let surface = font.render(text).solid(color).map_err(|e| e.to_string())?;

    let creator = canvas.texture_creator();
    let texture = creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;
    let rect = if centered {
        let text_w = texture.query().width;
        let text_h = texture.query().height;
        Rect::new(
            origin.x - text_w as i32 / 2,
            origin.y - text_h as i32 / 2,
            text_w,
            text_h,
        )
    } else {
        Rect::new(
            origin.x,
            origin.y,
            texture.query().width,
            texture.query().height,
        )
    };

    canvas.set_draw_color(color);
    canvas.copy(&texture, None, rect)?;

    Ok(())
}
