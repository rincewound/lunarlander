
pub struct vec2d
{
    pub x: f32,
    pub y: f32
}

impl vec2d
{
    pub fn new(x: f32, y: f32) -> Self
    {
        vec2d{
            x,
            y
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::vecmath::vec2d;

    #[test]
    pub fn create_vec()
    {
        let x = vec2d::new(10.0, 20.0);
    }

}