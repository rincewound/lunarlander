
pub struct Vec2d
{
    pub x: f32,
    pub y: f32
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


#[cfg(test)]
mod tests {
    use crate::vecmath::Vec2d;

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
}