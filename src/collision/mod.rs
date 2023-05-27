use crate::graphics;
use crate::vecmath::*;
use geo::Contains;
use geo::LineString;
use geo::coord;
use geo::polygon;
use sdl2::rect::Rect;
use sdl2::rect::Point;


fn is_on_segment(a: Vec2d, b: Vec2d, sample: Vec2d) -> bool
{
    let total = (a - b).len();
    let dA = (a-sample).len();
    let dB = (b-sample).len();
    return ((dA + dB) - total).abs() < 0.001
}

fn make_linestring(points: &Vec<Vec2d>) -> LineString
{
    let coords = points.iter().map(|v| coord!(x : v.x as f64, y: v.y as f64)).collect();
    let mut ls = LineString::new(coords);
    ls
}

pub fn hit_test(vertex: Vec2d, body: &Vec<Vec2d>) -> bool
{
    let ls = make_linestring(body);
    let poly = geo::Polygon::new(ls, vec![]);
    let point = coord!{x:vertex.x as f64, y: vertex.y as f64};
    return poly.contains(&point);
}


/*
    Detect collision between a bbox and a list of lines (given by means of a pointlist)
 */
pub fn detect_collision(bbox: Vec<Vec2d>, points: &Vec<Vec2d>)
    -> Option<Vec<(Vec2d, Vec2d)>>
{
    // create vector of bounding box lines
    let mut bb_lines: Vec<(Vec2d, Vec2d)> = Vec::new();
    for idx in 1..bbox.len() {
        bb_lines.push((bbox[idx-1], bbox[idx]));
    }
    bb_lines.push((bbox[bbox.len()-1], bbox[0]));

    let mut collisions: Vec<(Vec2d, Vec2d)> = Vec::new();

    for idx in 1..points.len() {
        for (a,b) in bb_lines.iter() {
            if let Some(collision_point) = get_line_intersection(*a, *b, points[idx - 1], points[idx]) {
                if !is_on_segment(*a, *b, collision_point) {continue;}
                if !is_on_segment(points[idx - 1], points[idx], collision_point) {continue;}               
                collisions.push((points[idx - 1], points[idx]));
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

    use crate::collision::{self, hit_test};
    use crate::vecmath::Vec2d;

    #[test]
    fn test_hittest()
    {
        let p1 = Vec2d::new(0.0, 0.0);
        let p2 = Vec2d::new(2.0, 0.0);
        let p3 = Vec2d::new(2.0, 2.0);
        let p4 = Vec2d::new(0.0, 2.0);

        let body = vec![p1, p2, p3, p4];
        let v0 = Vec2d::new(1.0, 1.0);

        assert!(hit_test(v0, &body));
    }

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