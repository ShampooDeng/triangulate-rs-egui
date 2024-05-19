#![warn(clippy::all, rust_2018_idioms)]

//https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html#best-practices-for-packages-with-a-binary-and-a-library
mod app;
pub use app::Painting;

#[cfg(test)]
mod tests {
    use egui::pos2;

    #[test]
    fn test_kdtree() {
        let points = vec![pos2(1., 1.), pos2(2., 2.), pos2(3., 1.)];
        let kdtree = kd_tree::KdTree2::build_by_ordered_float(
            Vec::from_iter(points.iter().map(|point| [point.x, point.y]))
        );
        assert_eq!(kdtree.nearest(&[1.,1.1]).unwrap().item, &[1.,1.]);
    }
}
