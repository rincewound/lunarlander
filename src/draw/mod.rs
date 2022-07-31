use sdl2::pixels::Color;
pub use sdl2::rect::Point;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf;
use sdl2::video::Window;
use std::path::Path;

use crate::vecmath::Vec2d;

pub fn draw_line(
    canvas: &mut Canvas<Window>,
    from: &Vec2d,
    to: &Vec2d,
    color: Color,
) -> Result<(), String>
{
    canvas.set_draw_color(color);
    return canvas.draw_line(Point::new(from.x as i32, from.y as i32), Point::new(to.x as i32, to.y as i32));
}

pub fn draw_lines(
    canvas: &mut Canvas<Window>,
    points: &Vec<Vec2d>,
    color: Color,
    close: bool,
) -> Result<(), String>
{
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

pub fn draw_rect<T>(
    canvas: &mut Canvas<Window>,
    origin: &Vec2d,
    width: u32,
    height: u32,
    color: Color,
    fill: bool,
) -> Result<(), String>
{
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
