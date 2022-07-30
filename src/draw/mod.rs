use sdl2::pixels::Color;
pub use sdl2::rect::Point;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::ttf;
use sdl2::video::Window;
use std::path::Path;

pub trait Drawable {
    fn to_point(&self) -> Point;
}

pub fn draw_line<T>(
    canvas: &mut Canvas<Window>,
    from: &T,
    to: &T,
    color: Color,
) -> Result<(), String>
where
    T: Drawable,
{
    canvas.set_draw_color(color);
    return canvas.draw_line(from.to_point(), to.to_point());
}

pub fn draw_lines<T>(
    canvas: &mut Canvas<Window>,
    points: &Vec<Box<T>>,
    color: Color,
    close: bool,
) -> Result<(), String>
where
    T: Drawable,
{
    canvas.set_draw_color(color);

    for idx in 1..points.len() {
        if let Err(error) = draw_line(canvas, &*points[idx - 1], &*points[idx], color) {
            return Err(error);
        }
    }

    if close {
        if let Err(error) = draw_line(canvas, &*points[points.len() - 1], &*points[0], color) {
            return Err(error);
        }
    }

    Ok(())
}

pub fn draw_rect<T>(
    canvas: &mut Canvas<Window>,
    origin: &T,
    width: u32,
    height: u32,
    color: Color,
    fill: bool,
) -> Result<(), String>
where
    T: Drawable,
{
    canvas.set_draw_color(color);
    let origin_as_point = origin.to_point();
    let rect = Rect::new(origin_as_point.x, origin_as_point.y, width, height);

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
    let ttf_context = sdl2::ttf::init().map_err(|e| e.to_string())?;

    let font_path = Path::new("assets/DejaVuSansMono.ttf");
    let font = ttf_context.load_font(font_path, font_size)?;

    let surface = font
        .render(text)
        .solid(color)
        .map_err(|e| e.to_string())?;

    let creator = canvas.texture_creator();
    let texture = creator
        .create_texture_from_surface(&surface)
        .map_err(|e| e.to_string())?;
    let rect = Rect::new(origin.x, origin.y, texture.query().width, texture.query().height);

    canvas.set_draw_color(color);
    canvas.copy(&texture, None, rect)?;

    Ok(())
}
