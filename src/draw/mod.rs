use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
pub use sdl2::rect::Point;

pub trait Drawable
{
    fn to_point(&self) -> Point;
}

pub fn draw_line<T>(canvas: &mut Canvas<Window>, from: &T, to: &T, 
    color: Color) -> Result<(), String>
    where T: Drawable
{
    canvas.set_draw_color(color);
    return canvas.draw_line(from.to_point(), to.to_point());
}

pub fn draw_lines<T>(canvas: &mut Canvas<Window>, points: &Vec<Box<T>>, color: Color)
    -> Result<(), String>
    where T: Drawable
{
    canvas.set_draw_color(color);

    for idx in 1..points.len() {
        if let Err(error) = draw_line(canvas, &*points[idx-1], &*points[idx], color) {
            return Err(error);
        }
    }

    Ok(())
}

pub fn draw_rect<T>(canvas: &mut Canvas<Window>, origin: &T, width: u32, height: u32, color: Color,
    fill: bool)
    -> Result<(), String>
    where T: Drawable
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