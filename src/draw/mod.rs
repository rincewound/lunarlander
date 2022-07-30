use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::pixels::Color;
pub use sdl2::rect::Point;

pub trait Drawable
{
    fn to_point(&self) -> Point;
}

pub fn draw_line<T>(canvas: &mut Canvas<Window>, from: T, to: T, 
    color: Color) -> Result<(), String>
    where T: Drawable
{
    canvas.set_draw_color(color);
    return canvas.draw_line(from.to_point(), to.to_point());
}

//pub fn draw_lines<T>(canvas: &mut Canvas<Window>, points: &Vec<Box<T>>, color: Color)
//    -> Result<(), String>
//    where T: Drawable
//{
//    canvas.set_draw_color(color);
//    return canvas.draw_lines::<&[sdl2::rect::Point]>(&points.iter().map(|p| { p.to_point()}).collect());
//}