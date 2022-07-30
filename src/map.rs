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
            values: Vec::from([
                Vec2d::new(0.0, randomY(0.0, maxY)),
                Vec2d::new(maxX, randomY(0.0, maxY)),
            ]),
        }
    }

    fn sort(self: &mut Self) {
        self.values.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
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

#[cfg(test)]
mod tests {
    use crate::map::*;

    #[test]
    fn test_list_gen() {
        let li = PointList::new(10.0, 20.0);
        assert_eq!(li.values.len(), 2);
    }

    #[test]
    fn test_split() {
        let mut list = PointList::new(100.0, 100.0);
        split(list.values[0], list.values[1], &mut list.values, 5.0, 20.0);
        println!("Point list: {:?}", list);
    }

    #[test]
    fn test_sort_after_split() {
        let mut list = PointList::new(100.0, 100.0);
        split(list.values[0], list.values[1], &mut list.values, 5.0, 20.0);
        list.sort();
        println!("Point list (sorted): {:?}", list);
        for idx in 1..list.values.len() {
            assert_eq!(list.values[idx - 1].x < list.values[idx].x, true);
        }
    }
}
