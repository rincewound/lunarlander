use crate::graphics;
use crate::vecmath::*;
use sdl2::rect::Rect;
use sdl2::rect::Point;


pub fn detect_collision(position: &Vec2d, direction: &Vec2d, points: &Vec<Vec2d>)
    -> Option<Vec<(Vec2d, Vec2d)>>
{
    let scale = TransformationMatrix::scale(graphics::LanderScale.x, graphics::LanderScale.y);
    let origin = Vec2d::new(graphics::LanderBoundBoxOrigin.x + (position.x - graphics::LanderWidth as f32),
        graphics::LanderBoundBoxOrigin.y + (position.y - graphics::LanderHeight as f32));
    let scaled_origin = scale.transform(&origin);

    let bounding_rect = Rect::new(
        origin.x as i32,
        origin.y as i32,
        graphics::LanderWidth * graphics::LanderScale.x as u32,
        graphics::LanderHeight * graphics::LanderScale.y as u32);

    let mut collisions: Vec<(Vec2d, Vec2d)> = Vec::new();

    for idx in 1..points.len() {
        let start = Point::new(points[idx - 1].x as i32, points[idx - 1].y as i32);
        let end = Point::new(points[idx].x as i32, points[idx].y as i32);

        if let Some(collision_points) = bounding_rect.intersect_line(start, end) {
            collisions.push((Vec2d::new(
                collision_points.0.x as f32, 
                collision_points.0.y as f32),
                Vec2d::new(
                collision_points.1.x as f32, 
                collision_points.1.y as f32)));
        }
    }

    if collisions.len() > 0 {
        return Some(collisions);
    } else {
        return None;
    }
}

#[cfg(test)]
mod tests {

}