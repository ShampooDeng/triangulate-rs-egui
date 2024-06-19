#![warn(clippy::all, rust_2018_idioms)]

//https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html#best-practices-for-packages-with-a-binary-and-a-library
mod app;
mod monotone_triangulation;
mod monotone_y_partition;
mod transform_pos;
mod triangle_base;
mod vertex_coloring;

pub use app::Painting;

pub const NIL: usize = !0;

pub trait Circulator {
    fn prev(&self, idx: usize) -> usize;
    fn next(&self, idx: usize) -> usize;
}

impl Circulator for Vec<usize> {
    fn next(&self, idx: usize) -> usize {
        self[(idx + 1) % self.len()]
    }

    fn prev(&self, idx: usize) -> usize {
        if idx == 0 {
            return self[self.len() - 1];
        }
        self[idx - 1]
    }
}

// impl Circulator for [usize] {
//     fn next(&self, idx: usize) -> usize {
//         (idx + 1) % self.len()
//     }

//     fn prev(&self, idx: usize) -> usize {
//         if idx == 0 {
//             return self.len() - 1;
//         }
//         idx - 1
//     }
// }
