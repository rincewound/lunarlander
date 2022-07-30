use crate::vecmath::Vec2d;
use rand::distributions::Uniform;
use rand::prelude::*;

#[derive(Debug)]
pub struct PointList {
    values: Vec<Vec2d>,
}

impl PointList {
    pub fn new(maxX: f32, maxY: f32) -> Self {
        PointList {
            values: Vec::from([Vec2d::new(0.0, 0.0), Vec2d::new(maxX, maxY)]),
        }
    }
}

fn randomY(minValue: f32, maxValue: f32) -> f32 {
    let mut rng = rand::thread_rng();
    let distY = Uniform::new_inclusive(minValue, maxValue);
    rng.sample(distY)
}

fn split(a: Vec2d, b: Vec2d, list: &mut Vec<Vec2d>, xMinDist: f32, yMaxDelta: f32) {
    assert_eq!(a.x < b.x, true);
    let deltaX = (b.x - a.x) / 2.0;
    let center = (a + b) / 2.0;
    let newY = randomY(center.y - (yMaxDelta / 2.0), center.y + (yMaxDelta / 2.0));
    let newPoint = Vec2d::new(a.x + deltaX, newY);
    if deltaX > xMinDist {
        split(a, newPoint, list, xMinDist, yMaxDelta / 1.5);
        split(newPoint, b, list, xMinDist, yMaxDelta / 1.5);
    }
    list.push(newPoint);
}

fn interpolate(a: f32, b: f32, w: f32) -> f32 {
    if w < 0.0 {
        a
    } else if w > 1.0 {
        b
    } else {
        (b - a) * w + a
    }
}

pub fn interpolate2d(a: &Vec2d, b: &Vec2d, maxYdelta: f32) -> Vec2d {
    let x = interpolate(a.x, b.x, 0.5);
    let y = interpolate(a.y, b.y, 0.5);
    Vec2d::new(x, y)
}

#[cfg(test)]
mod tests {
    use crate::map::*;

    #[test]
    fn test_list_gen() {
        let li = PointList::new(10.0, 20.0);
        assert_eq!(li.values.len(), 2);
    }
    #[test]
    fn test_interpolate() {
        let x = interpolate(5.0, 10.0, 0.5);
        assert_eq!(x, 7.5);
        let x = interpolate(8.0, 12.0, 0.25);
        assert_eq!(x, 9.0);
    }

    #[test]
    fn test_interpolat2d() {
        let v = interpolate2d(&Vec2d::new(5.0, 5.0), &Vec2d::new(9.0, 9.0), 0.5);
        assert_eq!(v.x, 7.0);
        assert_eq!(v.y, 7.0);
    }

    #[test]
    fn test_split() {
        let mut list = PointList::new(100.0, 100.0);
        //let start =
        split(list.values[0], list.values[1], &mut list.values, 5.0, 20.0);
        println!("Point list: {:?}", list);
    }
}
