use crate::vecmath::Vec2d;

#[derive(Debug)]
pub struct PointList {
    values: Vec<Vec2d>,
}

impl PointList {
    pub fn new() -> Self {
        PointList {
            values: Vec::from([Vec2d::new(0.0, 5.0), Vec2d::new(100.0, 10.0)]),
        }
    }

    pub fn addPoint(self: &mut Self) {
        for idx in 0..self.values.len() - 1 {
            let newPoint = interpolate2d(&self.values[idx], &self.values[idx + 1], 0.5);
            self.values.insert(idx + 1, newPoint);
        }
    }
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

pub fn interpolate2d(a: &Vec2d, b: &Vec2d, w: f32) -> Vec2d {
    let x = interpolate(a.x, b.x, 0.5);
    let y = interpolate(a.y, b.y, 0.5);
    Vec2d::new(x, y)
}

#[cfg(test)]
mod tests {
    use crate::map::*;

    #[test]
    fn test_list_gen() {
        let li = PointList::new();
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
    fn test_addPoint() {
        let mut li = PointList::new();
        li.addPoint();
        assert_eq!(li.values.len(), 3);
        li.addPoint();
        assert_eq!(li.values.len(), 5);
        li.addPoint();
        assert_eq!(li.values.len(), 9);
        println!("Point list: {:?}", li);
    }
}
