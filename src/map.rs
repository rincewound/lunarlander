use crate::vecmath::Vec2d;
use rand::distributions::Uniform;
use rand::prelude::*;

#[derive(Debug)]
pub struct PointList {
    values: Vec<Vec2d>,
}

const X_MAX_DELTA: f32 = 10.0;
const Y_MAX_DELTA: f32 = 200.0;
const Y_DELTA_DIVIDOR: f32 = 1.70;
const X_START_POINTS: usize = 10;

impl PointList {
    pub fn new(maxX: f32, maxY: f32) -> Self {
        let mut start_points: Vec<Vec2d> = Vec::new();
        for val in (0..=(maxX as usize)).step_by((maxX as usize) / X_START_POINTS) {
            let val = val as f32;
            println!("Start point val: {}", val);
            start_points.push(Vec2d::new(val, randomY(0.0, maxY)))
        }
        let mut gen_map = Vec::new();

        gen_map.push(start_points.first().unwrap().clone());
        for idx in 1..start_points.len() {
            gen_map.push(start_points[idx].clone());
            split(
                start_points[idx - 1].clone(),
                start_points[idx].clone(),
                &mut gen_map,
                X_MAX_DELTA,
                Y_MAX_DELTA,
            );
        }

        let mut n = PointList {
            values: Vec::from(gen_map),
        };
        n.sort();
        n
    }

    pub fn get_values(self: &Self) -> &Vec<Vec2d> {
        &self.values
    }

    fn sort(self: &mut Self) {
        self.values.sort_by(|a, b| a.x.partial_cmp(&b.x).unwrap());
    }
}

fn randomY(minValue: f32, maxValue: f32) -> f32 {
    loop {
        let mut rng = rand::thread_rng();
        let distY = Uniform::new_inclusive(minValue, maxValue);
        let num = rng.sample(distY);
        if num >= 0.0 {
            return num;
        } else {
            println!("randomY got illegal value below 0");
        }
    }
}

fn split(a: Vec2d, b: Vec2d, list: &mut Vec<Vec2d>, xMinDist: f32, yMaxDelta: f32) {
    assert_eq!(a.x < b.x, true);
    let deltaX = (b.x - a.x) / 2.0;
    let center = (a + b) / 2.0;
    let newY = randomY(center.y - (yMaxDelta / 2.0), center.y + (yMaxDelta / 2.0));
    let newPoint = Vec2d::new(a.x + deltaX, newY);
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
    const X_SIZE: f32 = 800.0;
    const Y_SIZE: f32 = 400.0;

    #[test]
    fn test_list_gen() {
        let li = PointList::new(10.0, 20.0);
        println!("Point list: {:?}", li);
        assert_eq!(li.values.first().unwrap().x == 0.0, true);
        assert_eq!(li.values.last().unwrap().x == 10.0, true);
    }

    #[test]
    fn test_split() {
        let mut list = Vec::from([
            Vec2d::new(0.0, randomY(0.0, 100.0)),
            Vec2d::new(100.0, randomY(0.0, 100.0)),
        ]);
        split(list[0].clone(), list[1].clone(), &mut list, 5.0, 20.0);
        println!("Point list: {:?}", list);
    }

    #[test]
    fn test_list_is_sortet_in_x_direction() {
        let list = PointList::new(X_SIZE, Y_SIZE);
        println!("Point list (sorted): {:?}", list);
        for idx in 1..list.values.len() {
            assert_eq!(list.values[idx - 1].x < list.values[idx].x, true);
        }
    }

    #[test]
    fn test_point_list_y_is_non_negatie() {
        let list = PointList::new(X_SIZE, Y_SIZE);
        for val in list.values.iter() {
            assert_eq!(val.y > 0.0, true);
        }
    }
}
