use std::{ops::Mul, time::Duration};

use glam::{vec2, Vec2};

use crate::solver::Constraint;

#[derive(Debug, Clone)]
pub struct Particle {
    pub radius: f32,
    pub pos: glam::Vec2,
    pub pos_old: glam::Vec2,
    pub acc: glam::Vec2,
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            radius: 0.5,
            pos: glam::Vec2::ZERO,
            pos_old: glam::Vec2::ZERO,
            acc: glam::Vec2::ZERO,
        }
    }
}

impl Particle {
    const GRAVITY : Vec2 = vec2(0., -0.03);

    pub fn new(radius: f32, pos: Vec2) -> Self {
        Self {
            radius,
            pos,
            pos_old: pos,
            acc: glam::Vec2::ZERO,
        }
    }

    pub fn update(&mut self, dt: f32) {
        let new_pos = self.pos*2. - self.pos_old + self.acc*dt*dt;
        self.pos_old = self.pos;
        self.set_position(new_pos, false);
    }

    pub fn apply_gravity(&mut self) {
        self.apply_force(Particle::GRAVITY);
    }

    pub fn apply_force(&mut self, force: Vec2) {
        self.acc += force;
    }

    pub fn set_position(&mut self, pos: Vec2, keep_acc: bool) {
        //self.pos_old = self.pos;
        self.pos = pos;
        self.acc = if keep_acc {self.acc} else {Vec2::ZERO};
    }

    pub fn apply_constraint(&mut self, constraint: Constraint) {
        match constraint {
            Constraint::Cup(bl, tr) => {
                let new_x = self.pos.x.max(bl.x + self.radius).min(tr.x - self.radius);
                let new_y = self.pos.y.max(bl.y + self.radius);
                if (new_x, new_y) != (self.pos.x, self.pos.y) {
                    self.set_position(vec2(new_x, new_y), false);
                }
            },
            Constraint::Box(bl, tr) => {
                let new_x = self.pos.x.max(bl.x + self.radius).min(tr.x - self.radius);
                let new_y = self.pos.y.max(bl.y + self.radius).min(tr.y - self.radius);
                if (new_x, new_y) != (self.pos.x, self.pos.y) {
                    self.set_position(vec2(new_x, new_y), false);
                }
            },
        }
    }
}

