use crate::vecmath::Vec2d;
use rand::distributions::Uniform;
use rand::prelude::*;

#[derive(Debug)]
pub struct PointList {
    values: Vec<Box<Vec2d>>,
}

const X_MAX_DELTA: f32 = 10.0;
const Y_MAX_DELTA: f32 = 200.0;
const Y_DELTA_DIVIDOR: f32 = 1.70;

impl PointList {
    pub fn new(maxX: f32, maxY: f32) -> Self {
        let mut n = PointList {
            values: Vec::from([
                Box::new(Vec2d::new(0.0, randomY(0.0, maxY))),
                Box::new(Vec2d::new(maxX, randomY(0.0, maxY))),
            ]),
        };
        split(
            n.values[0].clone(),
            n.values[1].clone(),
            &mut n.values,
            X_MAX_DELTA,
            Y_MAX_DELTA,
        );
        n.sort();
        n
    }

    pub fn get_values(self: &Self) -> &Vec<Box<Vec2d>> {
        &self.values
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

fn split(a: Box<Vec2d>, b: Box<Vec2d>, list: &mut Vec<Box<Vec2d>>, xMinDist: f32, yMaxDelta: f32) {
    assert_eq!(a.x < b.x, true);
    let deltaX = (b.x - a.x) / 2.0;
    let center = (*a + *b) / 2.0;
    let newY = randomY(center.y - (yMaxDelta / 2.0), center.y + (yMaxDelta / 2.0));
    let newPoint = Box::new(Vec2d::new(a.x + deltaX, newY));
    if deltaX > xMinDist {
        split(
            a,
            newPoint.clone(),
            list,
            xMinDist,
            yMaxDelta / Y_DELTA_DIVIDOR,
        );
        split(
            newPoint.clone(),
            b,
            list,
            xMinDist,
            yMaxDelta / Y_DELTA_DIVIDOR,
        );
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
        split(
            list.values[0].clone(),
            list.values[1].clone(),
            &mut list.values,
            5.0,
            20.0,
        );
        println!("Point list: {:?}", list);
    }

    #[test]
    fn test_sort_after_split() {
        let mut list = PointList::new(100.0, 100.0);
        split(
            list.values[0].clone(),
            list.values[1].clone(),
            &mut list.values,
            5.0,
            20.0,
        );
        list.sort();
        println!("Point list (sorted): {:?}", list);
        for idx in 1..list.values.len() {
            assert_eq!(list.values[idx - 1].x < list.values[idx].x, true);
        }
    }
}
