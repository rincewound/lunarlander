use std::f32::consts::PI;

#[allow(dead_code)]
use rand::Rng;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vec2d {
    pub x: f32,
    pub y: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct TransformationMatrix {
    m: [[f32; 3]; 3],
}

impl Vec2d {
    pub const fn new(x: f32, y: f32) -> Self {
        Vec2d { x, y }
    }

    pub fn from_angle(angle: f32) -> Self {
        return Vec2d {
            x: angle.cos(),
            y: angle.sin(),
        };
    }

    pub fn default() -> Self {
        Vec2d { x: 0.0, y: 0.0 }
    }

    pub fn len(&self) -> f32 {
        return (self.x * self.x + self.y * self.y).sqrt();
    }

    pub fn normalized(&self) -> Vec2d {
        let x = self.clone();
        return x / x.len();
    }

    pub fn angle(&self) -> f32 {
        let n = self.normalized();
        let norm = Vec2d::new(1.0, 0.0);

        let dot = (n.x * norm.x + n.y * norm.y);
        let div =
            (n.x.powf(2.0) + n.y.powf(2.0)).sqrt() * (norm.x.powf(2.0) + norm.y.powf(2.0)).sqrt();
        return (dot / div).acos();
    }

    pub fn angle_360(&self) -> f32 {
        // this is a workaround because angle does only work for the upper half
        // e.g. 0 .. 180 deg
        if self.y >= 0.0 {
            self.angle()
        } else {
            self.rotate(PI).angle() + PI
        }
    }

    pub fn rotate(&self, rel_rot: f32) -> Vec2d {
        let a = self.angle() + rel_rot;
        return Vec2d::from_angle(a);
    }

    pub fn random(min_x: f32, max_x: f32, min_y: f32, max_y: f32) -> Vec2d {
        let mut rnd = rand::thread_rng();
        return Vec2d::new(rnd.gen_range(min_x..max_x), rnd.gen_range(min_y..max_y));
    }
}

impl std::ops::Add<Vec2d> for Vec2d {
    type Output = Vec2d;

    fn add(self, rhs: Vec2d) -> Self::Output {
        Vec2d {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub<Vec2d> for Vec2d {
    type Output = Vec2d;

    fn sub(self, rhs: Vec2d) -> Self::Output {
        Vec2d {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Mul<f32> for Vec2d {
    type Output = Vec2d;

    fn mul(self, rhs: f32) -> Self::Output {
        Vec2d {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::ops::Div<f32> for Vec2d {
    type Output = Vec2d;

    fn div(self, rhs: f32) -> Self::Output {
        Vec2d {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl TransformationMatrix {
    pub fn unit() -> Self {
        TransformationMatrix {
            m: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    pub fn translation(x: f32, y: f32) -> Self {
        TransformationMatrix {
            m: [[1.0, 0.0, x], [0.0, 1.0, y], [0.0, 0.0, 1.0]],
        }
    }

    pub fn translation_v(v: Vec2d) -> TransformationMatrix {
        Self::translation(v.x, v.y)
    }

    pub fn rotation_v(v: Vec2d) -> TransformationMatrix {
        Self::rotate(v.angle())
    }

    pub fn rotate(angle: f32) -> Self {
        TransformationMatrix {
            m: [
                [angle.cos(), -angle.sin(), 0.0],
                [angle.sin(), angle.cos(), 0.0],
                [0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn scale(sx: f32, sy: f32) -> Self {
        TransformationMatrix {
            m: [[sx, 0.0, 0.0], [0.0, sy, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    pub fn transform(&self, v: &Vec2d) -> Vec2d {
        let x = self.m[0][0] * v.x + self.m[0][1] * v.y + self.m[0][2] * 1.0;
        let y = self.m[1][0] * v.x + self.m[1][1] * v.y + self.m[1][2] * 1.0;
        Vec2d { x, y }
    }

    pub fn transform_many(&self, v: &Vec<Vec2d>) -> Vec<Vec2d> {
        let mut result: Vec<Vec2d> = Vec::new();
        for vector in v.iter() {
            result.push(self.transform(vector));
        }
        result
    }
}

impl std::ops::Mul<TransformationMatrix> for TransformationMatrix {
    type Output = TransformationMatrix;

    fn mul(self, rhs: TransformationMatrix) -> Self::Output {
        let v00 =
            self.m[0][0] * rhs.m[0][0] + self.m[0][1] * rhs.m[1][0] + self.m[0][2] * rhs.m[2][0];
        let v01 =
            self.m[0][0] * rhs.m[0][1] + self.m[0][1] * rhs.m[1][1] + self.m[0][2] * rhs.m[2][1];
        let v02 =
            self.m[0][0] * rhs.m[0][2] + self.m[0][1] * rhs.m[1][2] + self.m[0][2] * rhs.m[2][2];

        let v10 =
            self.m[1][0] * rhs.m[0][0] + self.m[1][1] * rhs.m[1][0] + self.m[1][2] * rhs.m[2][0];
        let v11 =
            self.m[1][0] * rhs.m[0][1] + self.m[1][1] * rhs.m[1][1] + self.m[1][2] * rhs.m[2][1];
        let v12 =
            self.m[1][0] * rhs.m[0][2] + self.m[1][1] * rhs.m[1][2] + self.m[1][2] * rhs.m[2][2];

        let v20 =
            self.m[2][0] * rhs.m[0][0] + self.m[2][1] * rhs.m[1][0] + self.m[2][2] * rhs.m[2][0];
        let v21 =
            self.m[2][0] * rhs.m[0][1] + self.m[2][1] * rhs.m[1][1] + self.m[2][2] * rhs.m[2][1];
        let v22 =
            self.m[2][0] * rhs.m[0][2] + self.m[2][1] * rhs.m[1][2] + self.m[2][2] * rhs.m[2][2];

        TransformationMatrix {
            m: [[v00, v01, v02], [v10, v11, v12], [v20, v21, v22]],
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use crate::vecmath::{TransformationMatrix, Vec2d};

    #[test]
    pub fn create_vec() {
        let _x = Vec2d::new(10.0, 20.0);
    }

    #[test]
    pub fn len_works() {
        let x = Vec2d::new(1.0, 1.0);
        assert_eq!(2.0_f32.sqrt(), x.len());
    }

    #[test]
    pub fn angle_works() {
        let v = Vec2d::new(1.0, 0.0);
        assert_eq!(v.angle(), 0.0);
        let v2 = Vec2d::new(0.0, 1.0);
        assert_eq!(v2.angle(), PI / 2.0);
    }

    #[test]
    pub fn vec_add_works() {
        let v = Vec2d::new(2.0, 3.0);
        let v2 = v + Vec2d::new(2.0, 3.0);
        assert_eq!(v2.x, 4.0);
        assert_eq!(v2.y, 6.0);
    }

    #[test]
    pub fn vec_sub_works() {
        let v = Vec2d::new(2.0, 3.0);
        let v2 = v - Vec2d::new(1.0, 1.0);
        assert_eq!(v2.x, 1.0);
        assert_eq!(v2.y, 2.0);
    }

    #[test]
    pub fn vec_mul_works() {
        let v = Vec2d::new(2.0, 3.0);
        let v2 = v * 2.0;
        assert_eq!(v2.x, 4.0);
        assert_eq!(v2.y, 6.0);
    }

    #[test]
    pub fn vec_div_works() {
        let v = Vec2d::new(4.0, 6.0);
        let v2 = v / 2.0;
        assert_eq!(v2.x, 2.0);
        assert_eq!(v2.y, 3.0);
    }

    #[test]
    pub fn vec_div_works_scale() {
        let v = Vec2d::new(4.0, 6.0);
        let v2 = v / v.len();
        assert_eq!(1.0, v2.len())
    }

    #[test]
    pub fn translation_works() {
        let v = Vec2d::new(10.0, 15.0);
        let xfrom = TransformationMatrix::translation(2.0, 3.0);
        let res = xfrom.transform(&v);

        assert_eq!(res.x, 12.0);
        assert_eq!(res.y, 18.0);
    }

    #[test]
    pub fn rotation_works() {
        let v = Vec2d::new(1.0, 0.0);
        let xfrom = TransformationMatrix::rotate(PI / 2.0_f32);
        let res = xfrom.transform(&v);

        assert!(res.x.abs() <= 0.001);
        assert_eq!(res.y, 1.0);
    }

    #[test]
    pub fn can_mult_matrix() {
        let m1 = TransformationMatrix::unit();
        let m2 = TransformationMatrix::translation(1.0, 2.0);
        let m3 = m1 * m2;

        let v = Vec2d::new(3.0, 4.0);
        let v2 = m3.transform(&v);

        assert_eq!(v2.x, 4.0);
        assert_eq!(v2.y, 6.0);
    }

    #[test]
    pub fn scale_works() {
        let v = Vec2d::new(1.0, 1.0);
        let xfrom = TransformationMatrix::scale(2.0, 3.0);
        let res = xfrom.transform(&v);

        assert_eq!(res.x, 2.0);
        assert_eq!(res.y, 3.0);
    }
    #[test]
    pub fn rotation_past_180deg_works() {
        // Creates a vec pointing -1/0
        let v = Vec2d::from_angle(PI);
        assert_eq!(v.x, -1.0);
        assert!(v.y.abs() < 0.001);

        // rotate by 90°, resulting in 0/-1
        let v2 = v.rotate(PI / 2.0);
        assert_eq!(v2.y, -1.0);
        assert!(v2.x.abs() < 0.001);
    }

    #[test]
    pub fn rotation_past_360deg_works() {
        // Creates a vec pointing -1/0
        let v = Vec2d::from_angle(2.0 * PI);
        assert_eq!(v.x, 1.0);
        assert!(v.y.abs() < 0.001);

        // rotate by 90°, resulting in 0/1
        let v2 = v.rotate(PI / 2.0);
        assert_eq!(v2.y, 1.0);
        assert!(v2.x.abs() < 0.001);
    }
}
