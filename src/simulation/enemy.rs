use crate::vecmath::Vec2d;

#[derive(Clone, Copy, PartialEq)]
pub enum EnemyType {
    Rect,
    Rombus,
    Wanderer,
    SpawningRect,
    MiniRect,
    Invalid,
}

#[derive(Clone)]
pub struct Enemy<'a> {
    pub entity_id: usize,
    pub ty: EnemyType,
    pub hull: &'a [Vec2d],
}

impl Enemy<'_> {
    pub fn get_score(&self) -> u32 {
        return match self.ty {
            EnemyType::Invalid => 0,
            EnemyType::Rombus => 100,
            EnemyType::Rect => 300,
            EnemyType::Wanderer => 200,
            EnemyType::SpawningRect => 200,
            EnemyType::MiniRect => 50,
        };
    }
}
