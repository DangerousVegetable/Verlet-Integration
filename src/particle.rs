use glam::Vec2;

#[derive(Debug, Clone)]
pub struct Particle {
    pub size: f32,
    pub pos: glam::Vec2,
    pub pos_old: glam::Vec2,
    pub acc: glam::Vec2,
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            size: 0.1,
            pos: glam::Vec2::ZERO,
            pos_old: glam::Vec2::ZERO,
            acc: glam::Vec2::ZERO,
        }
    }
}

impl Particle {
    pub fn new(size: f32, pos: Vec2) -> Self {
        Self {
            size,
            pos,
            pos_old: pos,
            acc: glam::Vec2::ZERO,
        }
    }
}

