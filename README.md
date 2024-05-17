# triangulate-rs-egui

This repository is a fork of [eframe_template](https://deps.rs/repo/github/emilk/eframe_template). This app is based on [egui's painting demo](https://github.com/emilk/egui/blob/master/crates/egui_demo_lib/src/demo/painting.rs).

The goal is to accomplish the task of my Computational Geometry course solely with rust.

This project is still working in progress, ^w^.

## Feature

* design polygon with mouse click
* polygon triangulate algorithm
* 3-coloring painting algorithm
* output log for debugging app
* illustrate the process of triangulating a monotone polygon step by step

## Technical details

>How to select a polygon vertex with mouse click?

The coordinates of a polygon's vertices can be stored in an ordered data structure, KD-Tree.
One can find a nearest vertex around the cursor by searching the nearest children with respect to cursor position in the KD-Tree.

Any available solution?

* [kd-tree](https://docs.rs/kd-tree/latest/kd_tree/) in rust crates
* [BTree-set](https://doc.rust-lang.org/std/collections/struct.BTreeSet.html) in rust's std collection

>How to implement a double ended queue?

There is already a [double-ended queue](https://doc.rust-lang.org/std/collections/struct.VecDeque.html) implemented with vec in rust's std collection.

>Which polygon triangulate algorithm to implement?

Currently, I only focus on triangulate a monotone polygon (because that's what I'm required to do :D).

[polygon triangulation on wikipedia](https://en.wikipedia.org/wiki/Polygon_triangulation)

>How to implement 3-coloring algorithm?

Let's do it recursively.

## Todo

* [x] design polygon with mouse click
* [ ] implement polygon triangulate algorithm
* [ ] implement 3-coloring painting algorithm
* [ ] output log for debugging app
* [ ] illustrate the process of `triangulate algorithm` step by step
* [ ] better ui experience
  * [ ] show operation hint, like warning, suggestion, etc.
  * [ ] show current on-going process
  * [ ] add acknowledgement page for the app
  * [ ] add a Genshin icon on acknowledgement page
