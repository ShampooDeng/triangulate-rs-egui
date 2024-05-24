#![warn(clippy::all, rust_2018_idioms)]

//https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html#best-practices-for-packages-with-a-binary-and-a-library
mod app;
pub use app::Painting;
