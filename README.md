# triangulate-rs-egui

This repository is a fork of [eframe_template](https://deps.rs/repo/github/emilk/eframe_template). This app is based on [egui's painting demo](https://github.com/emilk/egui/blob/master/crates/egui_demo_lib/src/demo/painting.rs).

The goal is to accomplish the task of my Computational Geometry course solely with rust.

This project is still working in progress, ^w^.

## Feature

* design polygon with mouse click
* monotone partition (sweep line algorithm) for simple polygon
* polygon triangulate
* 3-coloring triangle's vertices based on triangulation result
* choose any triangle inside polygon as startup triangle for 3-coloring
* ~~illustrate the process of triangulating a monotone polygon step by step~~

## Installation

Make sure you have the [Rust](https://www.rust-lang.org/tools/install) 1.78 installed on your machine. The latest stable version of Rust might also work, but I haven't tested yet🤔.

```shell
git clone https://github.com/ShampooDeng/triangulate-rs-egui
cd ./triangulate-rs-egui/
cargo build --release
```

## Technical details

>How to select a triangle partition inside polygon with mouse click?

The coordinates of a triangle's centroid can be stored in an ordered data structure, KD-Tree.
One can find a nearest vertex around the cursor by searching the nearest children with respect to cursor position in the KD-Tree.

Any available solution?

* [kd-tree](https://docs.rs/kd-tree/latest/kd_tree/) in rust crates
* [BTree-set](https://doc.rust-lang.org/std/collections/struct.BTreeSet.html) in rust's std collection

This is feature is actually implemented with crate kd-tree, see [app.rs](src/app.rs) for more details.

>How to partition a simple polygon into monotone ones?

[cgal](https://github.com/CGAL/cgal/blob/master/Partition_2/include/CGAL/Partition_2/partition_y_monotone_2.h) provides an implementation of monotone partition using sweep-line algorithm.

I've rewrite cgal's implementation in rust, see [triangulate_2.rs](src/triangulate_2.rs) for more details.

>Which polygon triangulate algorithm to implement?

There are ear-clip and sweep-line algorithm for polygon triangulation.
However, I only focus on sweep-line algorithm (because that's what I'm required to do :D).
The whole process will be:

1. monotone partition simple polygon drawn in counter-clock wise order
2. triangulate monotone partitions

* [polygon triangulation on wikipedia](https://en.wikipedia.org/wiki/Polygon_triangulation)

>How to implement 3-coloring algorithm?

Once the simple polygon is triangulated, one can use a data structure similar to [Doubly-Connected-Edge-List(DCEL)](https://www.cs.umd.edu/class/spring2020/cmsc754/Lects/lect10-dcel.pdf) to store a triangle's adjacencies, which will eventually result in a graph-like result(neighboring triangles faces are linked by their shared edges). After that, a vertex 3-coloring result can be derived by determine the vertex color of the startup triangle and then traversing triangle faces in the DCEL in a DFS manner.

>How to implement DCEL with rust?
Data inside DCEL might needs to be shared and mutable simultaneously, which will be hard to implement with Rust.
The way I do it is simply build a DCEL after the triangulation, basically build a static DCEL for the purpose of coloring vertices. In that way, I don't have to maintain a valid DCEL during the process polygon triangulation. Please go to [monotone_y_partition](src/monotone_y_partition.rs) for more details.

## Todo

* [x] design polygon with mouse click
* [x] implement monotone partition
* [x] implement polygon triangulate algorithm
* [x] implement 3-coloring painting algorithm
* [x] output log while debugging the app
* [ ] ~~illustrate the process of `triangulate algorithm` step by step~~
* [ ] better ui experience
  * [x] show operation hint, like warning, suggestion, etc.
  * [ ] ~~show current on-going process~~
  * [ ] add acknowledgement page for the app
  * [ ] add a Genshin(OvO) icon on acknowledgement page

## Thanks💖

Many thanks to following repository which inspired or helped me.

* [eframe_template](https://github.com/emilk/eframe_template)
* [CGAL](https://github.com/CGAL/cgal)
* [kdtree-rs](https://github.com/mrhooray/kdtree-rs)
