
#[derive(Copy, Clone)]
pub struct Vec2d
{
    pub x: f32,
    pub y: f32
}

#[derive(Copy, Clone)]
pub struct TransformationMatrix
{
    m: [[f32;3];3]
}

impl Vec2d
{
    pub fn new(x: f32, y: f32) -> Self
    {
        Vec2d{
            x,
            y
        }
    }

    pub fn len(&self) -> f32
    {
        return (self.x* self.x + self.y*self.y).sqrt();
    }
}

impl std::ops::Add<Vec2d> for Vec2d
{
    type Output = Vec2d;

    fn add(self, rhs: Vec2d) -> Self::Output {
        Vec2d
        {
            x: self.x + rhs.x,
            y: self.y + rhs.y
        }
    }
}

impl std::ops::Mul<f32> for Vec2d
{
    type Output = Vec2d;

    fn mul(self, rhs: f32) -> Self::Output {
        Vec2d
        {
            x: self.x * rhs,
            y: self.y * rhs
        }
    }
}

impl std::ops::Div<f32> for Vec2d
{
    type Output = Vec2d;

    fn div(self, rhs: f32) -> Self::Output {
        Vec2d
        {
            x: self.x / rhs,
            y: self.y / rhs
        }
    }
}


impl TransformationMatrix
{
    pub fn unit() -> Self
    {
        TransformationMatrix { m: 
            [
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0]
                ]
         }
    }

    pub fn translation(x: f32, y:f32) -> Self
    {
        TransformationMatrix { m: 
            [
                [1.0, 0.0, x],
                [0.0, 1.0, y],
                [0.0, 0.0, 1.0]
                ]
         }
    }

    pub fn rotate(angle: f32) ->  Self
    {
        TransformationMatrix { m: 
            [
                [angle.cos(), - angle.sin(), 0.0],
                [angle.sin(), angle.cos(), 0.0],
                [0.0, 0.0, 1.0]
                ]
         }
    }

    pub fn transform(&self, v: &Vec2d) -> Vec2d
    {
        let x = self.m[0][0] * v.x + self.m[0][1] * v.y + self.m[0][2] * 1.0;
        let y = self.m[1][0] * v.x + self.m[1][1] * v.y + self.m[1][2] * 1.0;
        Vec2d { x, y }
    }
}

impl std::ops::Mul<TransformationMatrix> for TransformationMatrix
{
    type Output = TransformationMatrix;

    fn mul(self, rhs: TransformationMatrix) -> Self::Output {

        let v00 = self.m[0][0] * rhs.m[0][0] + self.m[0][1] * rhs.m[1][0] + self.m[0][2] * rhs.m[2][0];
        let v01 = self.m[0][0] * rhs.m[0][1] + self.m[0][1] * rhs.m[1][1] + self.m[0][2] * rhs.m[2][1];
        let v02 = self.m[0][0] * rhs.m[0][2] + self.m[0][1] * rhs.m[1][2] + self.m[0][2] * rhs.m[2][2];

        let v10 = self.m[1][0] * rhs.m[1][0] + self.m[1][1] * rhs.m[1][0] + self.m[1][2] * rhs.m[1][0];
        let v11 = self.m[1][0] * rhs.m[1][1] + self.m[1][1] * rhs.m[1][1] + self.m[1][2] * rhs.m[1][1];
        let v12 = self.m[1][0] * rhs.m[1][2] + self.m[1][1] * rhs.m[1][2] + self.m[1][2] * rhs.m[1][2];

        let v20 = self.m[2][0] * rhs.m[2][0] + self.m[2][1] * rhs.m[2][0] + self.m[2][2] * rhs.m[2][0];
        let v21 = self.m[2][0] * rhs.m[2][1] + self.m[2][1] * rhs.m[2][1] + self.m[2][2] * rhs.m[2][1];
        let v22 = self.m[2][0] * rhs.m[2][2] + self.m[2][1] * rhs.m[2][2] + self.m[2][2] * rhs.m[2][2];

        TransformationMatrix
        {
            m: [
                [ v00, v01, v02],              
                [ v10, v11, v12],
                [ v20, v21, v22]
              ]
        }   
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use crate::vecmath::{Vec2d, TransformationMatrix};

    #[test]
    pub fn create_vec()
    {
        let _x = Vec2d::new(10.0, 20.0);
    }

    #[test]
    pub fn len_works()
    {
        let x = Vec2d::new(1.0, 1.0);
        assert_eq!(2.0_f32.sqrt(), x.len());
    }

    #[test]
    pub fn vec_add_works()
    {
        let v = Vec2d::new(2.0, 3.0);
        let v2 = v + Vec2d::new(2.0, 3.0);
        assert_eq!(v2.x, 4.0);
        assert_eq!(v2.y, 6.0);
    }

    #[test]
    pub fn vec_mul_works()
    {
        let v = Vec2d::new(2.0, 3.0);
        let v2 = v * 2.0;
        assert_eq!(v2.x, 4.0);
        assert_eq!(v2.y, 6.0);
    }

    #[test]
    pub fn vec_div_works()
    {
        let v = Vec2d::new(4.0,6.0);
        let v2 = v / 2.0;
        assert_eq!(v2.x, 2.0);
        assert_eq!(v2.y, 3.0);
    }

    
    #[test]
    pub fn vec_div_works_scale()
    {
        let v = Vec2d::new(4.0,6.0);
        let v2 = v / v.len();
        assert_eq!(1.0, v2.len())
    }

    #[test]
    pub fn translation_works()
    {
        let v = Vec2d::new(10.0, 15.0);
        let xfrom = TransformationMatrix::translation(2.0, 3.0);
        let res = xfrom.transform(&v);

        assert_eq!(res.x, 12.0);
        assert_eq!(res.y, 18.0);
    }

    #[test]
    pub fn rotation_works()
    {
        let v = Vec2d::new(1.0, 0.0);
        let xfrom = TransformationMatrix::rotate(PI / 2.0_f32);
        let res = xfrom.transform(&v);

        assert!(res.x.abs() <= 0.001);
        assert_eq!(res.y, 1.0);
    }

    #[test]
    pub fn can_mult_matrix()
    {
        let m1 = TransformationMatrix::unit();
        let m2 = TransformationMatrix::translation(1.0,2.0);
        let m3 = m1 * m2;

        let v = Vec2d::new(3.0,4.0);
        let v2 = m3.transform(&v);

        assert_eq!(v2.x, 4.0);
        assert_eq!(v2.y, 6.0);

    }

}