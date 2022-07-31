use crate::graphics;
use crate::vecmath::*;
use sdl2::rect::Rect;
use sdl2::rect::Point;
//use std::cmp::{min, max};


pub fn detect_collision(bbox: Vec<Vec2d>, points: &Vec<Vec2d>)
    -> Option<Vec<Vec2d>>
{
    // create vector of bounding box lines
    let mut bb_lines: Vec<(Vec2d, Vec2d)> = Vec::new();
    for idx in 1..bbox.len() {
        bb_lines.push((bbox[idx-1], bbox[idx]));
    }

    let mut collisions: Vec<Vec2d> = Vec::new();

    for idx in 1..points.len() {
        for (a,b) in bb_lines.iter() {
            if let Some(collision_point) = get_line_intersection(*a, *b, points[idx - 1], points[idx]) {
                if collision_point.x >= min(a.x, b.x) && collision_point.x <= max(a.x, b.x) &&
                    collision_point.y >= min(a.y, b.y) && collision_point.y <= max(a.y, b.y)
                {
                    collisions.push(collision_point);
                }
            }
        }
    }

    if collisions.len() > 0 {
        return Some(collisions);
    } else {
        return None;
    }
}

fn min(a: f32, b: f32) -> f32
{
    if a > b {
        b
    } else {
        a
    }
}

fn max(a: f32, b: f32) -> f32
{
    if a > b {
        a
    } else {
        b
    }
}

fn get_line_intersection(p1: Vec2d, p2: Vec2d, p3: Vec2d, p4: Vec2d) -> Option<Vec2d>
{
    let s1_x = p2.x - p1.x;
    let s2_x = p4.x - p3.x;
    let s1_y = p2.y - p1.y;
    let s2_y = p4.y - p3.y;

    let s = (-s1_y * (p1.x - p2.x) + s1_x * (p1.y - p3.y)) / (-s2_x * s1_y + s1_x * s2_y);
    let t = ( s2_x * (p1.y - p3.y) - s2_y * (p1.x - p3.x)) / (-s2_x * s1_y + s1_x * s2_y);

    if (s >= 0.0 && s <= 1.0 && t >= 0.0 && t <= 1.0)
    {
        // Collision detected
        return Some(Vec2d::new(p1.x + (t * s1_x), p1.y + (t * s1_y)));
    }

    return None
}


#[cfg(test)]
mod tests {

    use crate::collision;
    use crate::vecmath::Vec2d;

    #[test]
    fn test_line_intersection()
    {
        let p1 = Vec2d::new(0.0, 0.0);
        let p2 = Vec2d::new(10.0, 0.0);
        let p3 = Vec2d::new(5.0, -5.0);
        let p4 = Vec2d::new(5.0, 5.0);

        if let Some(result) = collision::get_line_intersection(p1, p2, p3, p4)
        {
            assert_eq!(result.x, 5.0);
            assert_eq!(result.y, 0.0);
        } else {
            assert!(false);
        }
    }
}