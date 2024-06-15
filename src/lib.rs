#![warn(clippy::all, rust_2018_idioms)]

//https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html#best-practices-for-packages-with-a-binary-and-a-library
mod app;
mod monotone_y_partition;
mod monotone_triangulation;
mod triangle_base;

// use std::collections::binary_heap::Iter;

pub use app::Painting;
use egui::{pos2, vec2, Pos2, Vec2};

pub const NIL: usize = !0;

pub struct TransformPos {
    pub translation: Vec2,
    pub scaling: Vec2,
}

impl Default for TransformPos {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl TransformPos {
    pub const IDENTITY: Self = Self {
        translation: Vec2::ZERO,
        scaling: Vec2::new(1., 1.),
    };

    pub fn new(translation: Vec2, scaling: Vec2) -> Self {
        Self {
            translation,
            scaling,
        }
    }

    pub fn inverse(&self) -> Self {
        Self::new(
            vec2(
                -self.translation.x / self.scaling.x,
                -self.translation.y / self.scaling.y,
            ),
            vec2(1. / self.scaling.x, 1. / self.scaling.y),
        )
    }

    pub fn mul_pos(&self, pos: Pos2) -> Pos2 {
        let ret = pos2(self.scaling.x * pos.x, self.scaling.y * pos.y);
        ret + self.translation
    }
}

impl std::ops::Mul<Pos2> for TransformPos {
    type Output = Pos2;

    fn mul(self, rhs: Pos2) -> Self::Output {
        let ret = pos2(self.scaling.x * rhs.x, self.scaling.y * rhs.y);
        ret + self.translation
    }
}
