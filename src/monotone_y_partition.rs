use crate::triangle_base::*;
use crate::Circulator;
use core::panic;
use egui::{Color32, Pos2};
use log::{debug, info};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::rc::Rc;

use MiddleVertexStatus::{Concave, Convex};
use Orientation::{ClockWise, CounterClockWise};

pub enum VertexType {
    StartVertex,
    EndVertex,
    RegularVetex,
    SplitVertex,
    MergeVertex,
}

// HACK: A copy of PartitionPolygon vertices,
// used as auxilary reference in sort_daig() and partition()
type Vertices = Vec<Pos2>;

pub struct PartitionVertex {
    // coordinates of point
    pub point: Pos2,
    // indexes of points on the other side of the diagonals
    pub diag_points: Vec<usize>,
    pub half_diags: Vec<Rc<RefCell<HalfDiag>>>,
    pub unused_diag_count: usize,
    pub _color: egui::Color32,
}

impl PartitionVertex {
    fn new(input: &Pos2) -> Self {
        PartitionVertex {
            point: *input, // Pos2 has copy trait, so just dereference it.
            diag_points: Vec::new(),
            half_diags: Vec::new(),
            unused_diag_count: 0,
            _color: Color32::BLACK,
        }
    }

    fn insert_diagonal(&mut self, vertex_idx: usize, half_diag: Rc<RefCell<HalfDiag>>) {
        if self.diag_points.iter().any(|&x| x == vertex_idx) {
            return;
        }
        self.diag_points.push(vertex_idx);
        self.half_diags.push(half_diag.clone());
        self.unused_diag_count += 1;
    }

    fn magnified_pos_x(&self) -> i32 {
        (self.point.x * 100.).round() as i32
    }

    /// Sort diagonals in ccw order, by their agnle relative to the line,\
    /// formed by current vertex and its' next vertex in polygon.
    fn sort_diag(&mut self, next: &Pos2, vertices: &Vertices) {
        self.diag_points.sort_by(|a, b| {
            let cur = &self.point;
            let vertex_1 = vertices[*a];
            let vertex_2 = vertices[*b];
            let angle_rad1 = compute_angle(cur, next, &vertex_1);
            let angle_rad2 = compute_angle(cur, next, &vertex_2);
            angle_rad1
                .partial_cmp(&angle_rad2)
                .expect("cmp is impossible")
        });
    }

    fn use_one_diag(&mut self) {
        self.unused_diag_count -= 1;
    }

    fn pop_current_diag(&mut self) -> usize {
        let ret = self.diag_points[self.unused_diag_count - 1];
        self.use_one_diag();
        ret
    }

    fn has_unused_diag(&self) -> bool {
        if self.unused_diag_count == 0 {
            return false;
        }
        true
    }

    fn reset_unused_diag_count(&mut self) {
        self.unused_diag_count = self.diag_points.len();
    }
}

pub struct HalfDiag {
    pub origin: usize,
    pub end: usize,
    pub twin: Option<Rc<RefCell<HalfDiag>>>,
    pub bounding_face: Option<Rc<RefCell<Face>>>,
}

impl HalfDiag {
    fn new(origin: usize, end: usize) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(HalfDiag {
            origin,
            end,
            twin: None,
            bounding_face: None,
        }))
    }

    fn spwan_twin(origin: usize, end: usize) -> (Rc<RefCell<Self>>, Rc<RefCell<Self>>) {
        let half_diag1 = HalfDiag::new(origin, end);
        let half_diag2 = HalfDiag::new(end, origin);
        half_diag1.borrow_mut().twin = Some(half_diag2.clone());
        half_diag2.borrow_mut().twin = Some(half_diag1.clone());
        (half_diag1, half_diag2)
    }
}

pub struct Face {
    pub vertices: Vec<usize>,
    pub centroid: Pos2,
    pub bounding_diags: Vec<Rc<RefCell<HalfDiag>>>,
}

impl Face {
    // pub fn new(vertices: Vec<usize>, coordinates: &Vertices) -> Self {
    //     let mut new_face = Face {
    //         vertices,
    //         centroid: Pos2::ZERO,
    //         bounding_diags: Vec::new(),
    //     };
    //     new_face.calc_centroid(coordinates);
    // }
    pub fn new(vertices: Vec<usize>, coordinates: &Vertices) -> Rc<RefCell<Face>> {
        let new_face = Rc::new(RefCell::new(Face {
            vertices,
            centroid: Pos2::ZERO,
            bounding_diags: Vec::new(),
        }));
        let _ = new_face.borrow_mut().calc_centroid(coordinates);
        new_face
    }

    fn calc_centroid(&mut self, coordinates: &Vertices) -> Result<(), ()> {
        if self.vertices.is_empty() {
            return Err(());
        }
        let mut x: f32 = 0.0;
        let mut y: f32 = 0.0;
        for vertex_idx in self.vertices.iter() {
            x += coordinates[*vertex_idx].x;
            y += coordinates[*vertex_idx].y;
        }
        let len = self.vertices.len() as f32;
        self.centroid = Pos2::new(x / len, y / len);
        Ok(())
    }
}

/// A simple polygon defined by a collection
/// of vertices in **ccw** order.
pub struct PartitionPolygon {
    pub vertices: Vec<PartitionVertex>,
    pub faces: Vec<Rc<RefCell<Face>>>,
    // pub faces: Vec<Face>,
}

impl Circulator for PartitionPolygon {
    /// Get previous vertex's index with respect to idx
    fn prev(&self, idx: usize) -> usize {
        if idx == 0 {
            return self.vertices.len() - 1;
        }
        idx - 1
    }

    /// Get next vertex's index with repsect to idx
    fn next(&self, idx: usize) -> usize {
        (idx + 1) % self.vertices.len()
    }
}

impl PartitionPolygon {
    pub fn new() -> Self {
        PartitionPolygon {
            vertices: Vec::new(),
            faces: Vec::new(),
        }
    }

    /// Insert diagonal between two vertices given
    /// their index in partition polygon
    pub fn insert_diagonal(&mut self, idx1: usize, idx2: usize) {
        if self.next(idx1) == idx2 || self.prev(idx1) == idx2 {
            return;
        }
        let (halfdiag1, halfdiag2) = HalfDiag::spwan_twin(idx1, idx2);
        info!("insert diagonal between {} and {}", idx1, idx2);
        self.vertices[idx1].insert_diagonal(idx2, halfdiag1);
        self.vertices[idx2].insert_diagonal(idx1, halfdiag2);
    }

    /// Build a partition polygon from a list of vertices
    /// in ccw order.
    pub fn build_from_pts(&mut self, input: &[Pos2]) {
        let input_iter = input.iter();
        for item in input_iter {
            self.vertices.push(PartitionVertex::new(item));
        }
    }

    pub fn reset_unused_diag_counts(&mut self) {
        info!("reset polygon vertices' diag count");
        for vertex in self.vertices.iter_mut() {
            vertex.reset_unused_diag_count();
        }
    }

    /// Use this method with rest_unused_diag_count()
    pub fn make_polygons(
        &mut self,
        start: usize,
        result: &mut Vec<Vec<usize>>,
        _vertices: &Vec<Pos2>,
    ) -> usize {
        let mut new_polygon: Vec<usize> = Vec::new();
        let mut idx: usize = start;
        debug!(
            "vertex{} has unused diag:{}",
            idx, self.vertices[idx].unused_diag_count
        );
        loop {
            new_polygon.push(idx);
            if self.vertices[idx].has_unused_diag() {
                let diag = self.vertices[idx].pop_current_diag();
                if diag == start {
                    debug!("push into result:{:?}", new_polygon);
                    result.push(new_polygon.clone());
                    return idx;
                } else {
                    idx = self.make_polygons(idx, result, _vertices);
                }
            } else {
                idx = (idx + 1) % self.vertices.len();
            }

            if idx == start && new_polygon.len() > 2 {
                debug!("push into result:{:?}", new_polygon);
                result.push(new_polygon.clone());
                break;
            }
        }
        idx
    }

    fn output_coordinates(&mut self, result: &[Vec<usize>]) -> Vec<Vec<Pos2>> {
        let mut result_pos: Vec<Vec<Pos2>> = Vec::new();
        for parition in result.iter() {
            let partition_coordinates = parition
                .iter()
                .map(|idx| self.vertices[*idx].point)
                .collect::<Vec<Pos2>>();
            result_pos.push(partition_coordinates);
        }
        result_pos
    }

    pub fn sort_diagonals(&mut self, vertices: &Vertices) {
        info!("sort polygon vertices' diag");
        for idx in 0..self.vertices.len() {
            if !self.vertices[idx].diag_points.is_empty() {
                let next_pos = self.vertices[(idx + 1) % self.vertices.len()].point;
                self.vertices[idx].sort_diag(&next_pos, vertices);
            }
            debug!("vertex{}'s diag: {:?}", idx, self.vertices[idx].diag_points);
        }
    }

    /// Output partitions described by a vector of vertices' coordinates
    pub fn partition(&mut self, vertices: &Vertices) -> Vec<Vec<Pos2>> {
        // HACK: change sort_diagonals's arg from PartitionVertex to Pos2,
        // since Vec<T> in PartitionVertex will cause multiple mutable
        // borrow of self, here, in this function.
        self.sort_diagonals(vertices);
        let mut result: Vec<Vec<usize>> = Vec::new();
        info!("---start making polygons---");
        let ret = self.make_polygons(0, &mut result, vertices);
        debug!("the return of make_polygons:{}", ret);
        self.reset_unused_diag_counts();
        self.link_face(&result, vertices);
        debug!("the num of polygon partition:{}", result.len());
        self.output_coordinates(&result)
    }

    fn link_face(&mut self, result: &[Vec<usize>], vertices: &Vec<Pos2>) {
        info!("---start link diag to face---");
        for partition in result.iter() {
            debug!("linking face{:?}", partition);
            let new_face = Face::new(partition.clone(), vertices);
            for i in 0..partition.len() {
                let point_idx = partition[i];
                if self.vertices[point_idx].diag_points.is_empty() {
                    debug!("vertex{} has {} diagognals", point_idx, 0);
                    continue;
                }
                let next_point_idx = partition.next(i);
                debug!(
                    "check vertex{} and its next vertex{}",
                    point_idx, next_point_idx
                );
                for half_diag in self.vertices[point_idx].half_diags.iter() {
                    if next_point_idx == half_diag.borrow().end {
                        debug!("found diag{}-{}", point_idx, next_point_idx);
                        new_face.borrow_mut().bounding_diags.push(half_diag.clone());
                        half_diag.borrow_mut().bounding_face = Some(new_face.clone());
                    }
                }
                self.faces.push(new_face.clone());
            }
        }
    }
}

struct PartitionTreeEntry {
    edge_origin: usize, // idx to edge's origin in PartitionPolygon
    helper: usize,      // idx to edge's helper in PartitionPlolygon
}

impl PartitionTreeEntry {
    fn new(origin_idx: usize, helper_idx: usize) -> Self {
        PartitionTreeEntry {
            edge_origin: origin_idx,
            helper: helper_idx,
        }
    }
}

impl Clone for PartitionTreeEntry {
    fn clone(&self) -> Self {
        PartitionTreeEntry {
            edge_origin: self.edge_origin,
            helper: self.helper,
        }
    }
}

struct PartitionTree {
    search_tree: BTreeMap<i32, PartitionTreeEntry>,
    keys: Vec<i32>,
}

impl PartitionTree {
    fn new() -> Self {
        PartitionTree {
            search_tree: BTreeMap::new(),
            keys: Vec::new(), // keys of binary tree map
        }
    }

    fn update_keys(&mut self) {
        self.keys = self.search_tree.clone().into_keys().collect::<Vec<i32>>();
    }

    /// Add a new edge and its helper by inserting its origin idx and its helper's idx.
    pub fn insert(&mut self, edge_origin_idx: usize, helper_idx: usize, poly: &PartitionPolygon) {
        debug!("tree before insert:{:?}", self.keys);
        let key = poly.vertices[edge_origin_idx].magnified_pos_x();
        self.search_tree
            .insert(key, PartitionTreeEntry::new(edge_origin_idx, helper_idx));
        self.update_keys();
        debug!("tree after insert:{:?}", self.keys);
    }

    /// Find the edge in the tree whose origin is edge_origin
    pub fn find(&self, edge_origin: &PartitionVertex) -> i32 {
        let vertex_pos_x = edge_origin.magnified_pos_x();
        match self.keys.binary_search(&vertex_pos_x) {
            Ok(a) => self.keys[a],
            _ => {
                panic!("can't find vertex in Tree")
            }
        }
    }

    /// Erase an edge from tree
    pub fn erase(&mut self, entry_key: i32) -> Result<(), i32> {
        debug!("tree before erase:{:?}", self.keys);
        if self.search_tree.remove(&entry_key).is_none() {
            return Err(entry_key);
        }
        self.update_keys();
        debug!("tree after erase:{:?}", self.keys);
        Ok(())
    }

    /// Find the a vertex's nearest neighbor in tree
    pub fn lower_bound(&self, vertex: &PartitionVertex, poly: &PartitionPolygon) -> i32 {
        // return 0 when search tree is empty
        if self.search_tree.is_empty() {
            return 0;
        }
        // let pred = vertex.magnified_pos_x();
        let low = self.keys.partition_point(|x| {
            indirect_edge_compare(self.search_tree.get(x).unwrap(), vertex, poly)
        }) - 1;
        self.keys[low]
    }
}

/// Compare the x coordinate of the most left vertex of a line with a vertex.
fn indirect_edge_compare(
    a: &PartitionTreeEntry,
    b: &PartitionVertex,
    poly: &PartitionPolygon,
) -> bool {
    let edge_origin_x = poly.vertices[a.edge_origin].point.x;
    let edge_end_x = poly.vertices[poly.next(a.edge_origin)].point.x;
    let most_left_vertex_x = if edge_origin_x.le(&edge_end_x) {
        edge_origin_x
    } else {
        edge_end_x
    };
    most_left_vertex_x.le(&b.point.x)
}

/// Get event vertex's left neighbor in the search tree
fn get_left_neighbor(
    vertex: &PartitionVertex,
    tree: &PartitionTree,
    poly: &PartitionPolygon,
) -> (i32, usize) {
    let key = tree.lower_bound(vertex, poly);
    (key, tree.search_tree[&key].helper)
}

/// Update an edge's helper in the search tree\
/// given it's origin's x coordinates(key)
fn update_helper(key: i32, new_helper: usize, tree: &mut PartitionTree) {
    if let Some(tree_entry) = tree.search_tree.get_mut(&key) {
        tree_entry.helper = new_helper;
    } else {
        panic!("can't update helper");
    }
}

/// Check a vertex's type by assessing their orientation
/// (ccw, cw) and its position with respect to its two neighbor vertices.
fn monoton_vertex_type(poly: &PartitionPolygon, idx: usize) -> VertexType {
    let prev: usize = poly.prev(idx);
    let next: usize = poly.next(idx);
    let p = &poly.vertices[prev].point;
    let q = &poly.vertices[idx].point;
    let r = &poly.vertices[next].point;
    match (cmp_slope(p, q, r), cmp_vertex_height(p, q, r)) {
        (CounterClockWise, Convex) => VertexType::StartVertex,
        (CounterClockWise, Concave) => VertexType::EndVertex,
        (ClockWise, Convex) => VertexType::SplitVertex,
        (ClockWise, Concave) => VertexType::MergeVertex,
        (_, _) => VertexType::RegularVetex,
    }
}

/// Generate event queue(it's actually works like a stack) of given vertices.\
/// All vertices are sorted by their y coordinates (from bottom to top, a.k.a, **incrementatl**).\
/// If vertices are at the same height, they will
/// be sorted by x coordinates (from right to left).
fn to_event_queue(input: &[PartitionVertex]) -> Vec<usize> {
    let mut output = Vec::from_iter(0..input.len());
    output.sort_by(|a, b| {
        let a_pos = input[*a].point;
        let b_pos = input[*b].point;
        let mut result = a_pos.y.partial_cmp(&b_pos.y).unwrap();
        if result.is_eq() {
            match a_pos.x.partial_cmp(&b_pos.x).unwrap() {
                Ordering::Greater => result = Ordering::Less,
                Ordering::Less => result = Ordering::Greater,
                _ => (),
            }
        }
        result
    });
    output
}

fn handle_start_vertex(vertex_idx: usize, tree: &mut PartitionTree, poly: &PartitionPolygon) {
    let edge_origin_idx = vertex_idx;
    let helper_idx = vertex_idx;
    tree.insert(edge_origin_idx, helper_idx, poly);
}

fn handle_end_vertex(vertex_idx: usize, tree: &mut PartitionTree, poly: &mut PartitionPolygon) {
    let prev = poly.prev(vertex_idx);
    let search_key = tree.find(&poly.vertices[prev]);
    let helper_prev_idx = tree.search_tree[&search_key].helper;
    if let VertexType::MergeVertex = monoton_vertex_type(poly, helper_prev_idx) {
        poly.insert_diagonal(vertex_idx, helper_prev_idx);
    }
    let _ = tree.erase(search_key);
}

/// Check if the polygon interior is in the right of **regular** vertex.\
/// Assuming all vertices are sorted in CCW order.
fn polygon_interior_to_right(vertex_idx: usize, poly: &PartitionPolygon) -> Result<bool, ()> {
    let prev: usize = poly.prev(vertex_idx);
    let next: usize = poly.next(vertex_idx);
    let p = &poly.vertices[prev].point;
    let q = &poly.vertices[vertex_idx].point;
    let r = &poly.vertices[next].point;
    match cmp_vertex_height(p, q, r) {
        MiddleVertexStatus::GradientDown => Ok(true),
        MiddleVertexStatus::GradientUp => Ok(false),
        _ => Err(()),
    }
}

fn handle_regular_vertex(vertex_idx: usize, tree: &mut PartitionTree, poly: &mut PartitionPolygon) {
    let interior_to_right = match polygon_interior_to_right(vertex_idx, poly) {
        Ok(res) => res,
        Err(_) => panic!("wrong middlevertex status"),
    };
    if interior_to_right {
        debug!("interior is right to vertex{}", vertex_idx);
        let prev = poly.prev(vertex_idx);
        let search_key = tree.find(&poly.vertices[prev]);
        let helper_prev_idx = tree.search_tree[&search_key].helper;
        if let VertexType::MergeVertex = monoton_vertex_type(poly, helper_prev_idx) {
            poly.insert_diagonal(vertex_idx, helper_prev_idx);
        }
        let _ = tree.erase(search_key);
        tree.insert(vertex_idx, vertex_idx, poly);
    } else {
        debug!("interior is left to vertex{}", vertex_idx);
        // BUG: If a regular vertex is colinear to a neighbor end vertex,
        // then "no entry found for key" or "attempt to subtract with overflow in line 364"
        let (left_neighbor_edge_key, left_neigbor_edge_helper) =
            get_left_neighbor(&poly.vertices[vertex_idx], tree, poly);
        if let VertexType::MergeVertex = monoton_vertex_type(poly, left_neigbor_edge_helper) {
            poly.insert_diagonal(vertex_idx, left_neigbor_edge_helper);
        }
        update_helper(left_neighbor_edge_key, vertex_idx, tree);
    }
}

fn handle_split_vertex(vertex_idx: usize, tree: &mut PartitionTree, poly: &mut PartitionPolygon) {
    let (left_neighbor_edge_key, left_neigbor_edge_helper) =
        get_left_neighbor(&poly.vertices[vertex_idx], tree, poly);
    poly.insert_diagonal(vertex_idx, left_neigbor_edge_helper);
    update_helper(left_neighbor_edge_key, vertex_idx, tree);
    tree.insert(vertex_idx, vertex_idx, poly);
}

fn handle_merge_vertex(vertex_idx: usize, tree: &mut PartitionTree, poly: &mut PartitionPolygon) {
    let prev = poly.prev(vertex_idx);
    let search_key = tree.find(&poly.vertices[prev]);
    let helper_prev_idx = tree.search_tree[&search_key].helper;
    if let VertexType::MergeVertex = monoton_vertex_type(poly, helper_prev_idx) {
        poly.insert_diagonal(vertex_idx, helper_prev_idx);
    }
    let _ = tree.erase(search_key);
    let (left_neighbor_edge_key, left_neigbor_edge_helper) =
        get_left_neighbor(&poly.vertices[vertex_idx], tree, poly);
    if let VertexType::MergeVertex = monoton_vertex_type(poly, left_neigbor_edge_helper) {
        poly.insert_diagonal(vertex_idx, left_neigbor_edge_helper);
    }
    update_helper(left_neighbor_edge_key, vertex_idx, tree);
}

/// Monotone partition a polygon by inserting diagonals in PartitionPolygon
/// NOTE: I can't figure out if it's the right way to define a function
/// that requires a &mut parameter.
pub fn monotone_partition(partition_poly: &mut PartitionPolygon) {
    let mut tree = PartitionTree::new();
    let mut event_queue = to_event_queue(&partition_poly.vertices);
    debug!("monotone partition event queue:{:?}", event_queue);
    while let Some(event_idx) = event_queue.pop() {
        match monoton_vertex_type(partition_poly, event_idx) {
            VertexType::StartVertex => {
                info!("vertex{} is start vertex", event_idx);
                handle_start_vertex(event_idx, &mut tree, partition_poly);
            }
            VertexType::EndVertex => {
                info!("vertex{} is end vertex", event_idx);
                handle_end_vertex(event_idx, &mut tree, partition_poly);
            }
            VertexType::RegularVetex => {
                info!("vertex{} is regular vertex", event_idx);
                handle_regular_vertex(event_idx, &mut tree, partition_poly);
            }
            VertexType::SplitVertex => {
                info!("vertex{} is split vertex", event_idx);
                handle_split_vertex(event_idx, &mut tree, partition_poly);
            }
            VertexType::MergeVertex => {
                info!("vertex{} is merge vertex", event_idx);
                handle_merge_vertex(event_idx, &mut tree, partition_poly);
            }
        }
    }
}

/// Monotone partition a polygon and output partitions' vertices coordinates.
pub fn monotone_polygon_partition(vertices: &Vec<Pos2>) -> Vec<Vec<Pos2>> {
    let mut partition_poly = PartitionPolygon::new();
    // let vertices_rc = vertices.iter().map(|x| Rc::new(x.clone()));
    partition_poly.build_from_pts(vertices);

    monotone_partition(&mut partition_poly);
    // Debug only
    // diagonals: 5<->3, 1<->3
    // assert_eq!(partition_poly.vertices[4].diag_points, Vec::new());
    // assert_eq!(partition_poly.vertices[6].diag_points, Vec::new());
    // assert_eq!(partition_poly.vertices[5].diag_points, vec![3]);
    // assert_eq!(partition_poly.vertices[3].diag_points, vec![5, 1]);
    // assert_eq!(partition_poly.vertices[1].diag_points, vec![3]);
    // assert_eq!(partition_poly.vertices[0].diag_points, Vec::new());
    // assert_eq!(partition_poly.vertices[2].diag_points, Vec::new());
    partition_poly.partition(vertices)
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::{
        monotone_polygon_partition, to_event_queue, PartitionPolygon, PartitionTree,
        PartitionVertex, VertexType,
    };
    use crate::monotone_y_partition::monoton_vertex_type;
    use egui::Pos2;

    #[test]
    fn test_build_from_pts() {
        let pts = vec![
            Pos2::new(1., 0.),
            Pos2::new(2., 1.),
            Pos2::new(2., 2.),
            Pos2::new(0., 1.),
        ];
        let mut poly = PartitionPolygon::new();
        poly.build_from_pts(&pts);
        let mut gt_iter = pts.iter();
        for item in poly.vertices.iter() {
            assert_eq!(item.point, *gt_iter.next().unwrap())
        }
    }

    fn init_poly() -> PartitionPolygon {
        /*
        Polygon example defined in counter clock-wise
                   4
        6         / \
        |\      /    \
        | \ 5 /       \
        |  \/          \3
        |     1        /
        |    /\      /
        |  /   \   /
        |/      \/
        0        2
        */
        let pts = vec![
            Pos2::new(1., 0.),
            Pos2::new(2., 1.),
            Pos2::new(3., 0.),
            Pos2::new(5., 1.5),
            Pos2::new(3.5, 3.),
            Pos2::new(1.5, 1.5),
            Pos2::new(1., 2.4),
        ];
        let mut poly = PartitionPolygon::new();
        poly.build_from_pts(&pts);
        poly
    }

    fn vertex_type_to_string(v_type: &VertexType) -> String {
        match v_type {
            VertexType::StartVertex => "start".to_string(),
            VertexType::EndVertex => "end".to_string(),
            VertexType::RegularVetex => "regular".to_string(),
            VertexType::SplitVertex => "split".to_string(),
            VertexType::MergeVertex => "merge".to_string(),
        }
    }

    #[test]
    fn test_event_queue() {
        let poly = init_poly();
        let event_queue = to_event_queue(&poly.vertices);
        assert_eq!(event_queue, vec![2, 0, 1, 3, 5, 6, 4]);
    }

    #[test]
    fn test_mono_vertex_type() {
        let poly = init_poly();
        let mut event_queue = to_event_queue(&poly.vertices);
        let mut results = Vec::new();
        loop {
            if event_queue.is_empty() {
                break;
            }
            let event_pt = event_queue.pop().unwrap();
            let ret = monoton_vertex_type(&poly, event_pt);
            results.push(ret);
        }

        let results_string = results
            .iter()
            .map(vertex_type_to_string)
            .collect::<Vec<String>>();
        let gts = vec!["start", "start", "merge", "regular", "split", "end", "end"];
        assert_eq!(results_string, gts);
    }

    #[test]
    fn test_lowerbound() {
        let poly = init_poly();
        let mut tree = PartitionTree::new();
        for idx in 3..7 {
            tree.insert(idx, idx, &poly);
        }
        assert_eq!(tree.keys, vec![100, 150, 350, 500]);
        // binary search for vertex(2., 1.)'s nearset left neighbor
        assert_eq!(tree.lower_bound(&poly.vertices[1], &poly), 350);
    }

    #[test]
    fn test_sort_diag() {
        let mut partition_vertex = PartitionVertex::new(&Pos2::new(10., 10.));
        partition_vertex.diag_points = (0..6).collect();
        let next = Pos2::new(12., 8.);
        let vertices = vec![
            Pos2::new(6., 7.),
            Pos2::new(4., 15.),
            Pos2::new(2., 10.),
            Pos2::new(10., 20.),
            Pos2::new(15., 10.),
            Pos2::new(8., 18.),
        ];
        partition_vertex.sort_diag(&next, &vertices);
        let res = partition_vertex.diag_points;
        let gts = vec![4, 3, 5, 1, 2, 0];
        assert_eq!(res, gts);
    }

    #[test]
    fn test_monotone_partition() {
        let pts = vec![
            Pos2::new(157., 29.),
            Pos2::new(308., 173.),
            Pos2::new(481., 49.),
            Pos2::new(624., 180.),
            Pos2::new(500., 349.),
            Pos2::new(378., 286.),
            Pos2::new(185., 333.),
        ];
        let result = monotone_polygon_partition(&pts);
        // assert_eq!(result[0], Vec::new());
        // assert_eq!(result[1], Vec::new());
        // assert_eq!(result[2], Vec::new());
        // assert_eq!(result.len() , 3);
        let mut res_iter = result.iter();
        assert_eq!(res_iter.next().unwrap(), &pts[1..4]);
        assert_eq!(res_iter.next().unwrap(), &pts[3..6]);
        let gt = vec![
            // 01356
            Pos2::new(157., 29.),
            Pos2::new(308., 173.),
            Pos2::new(624., 180.),
            Pos2::new(378., 286.),
            Pos2::new(185., 333.),
        ];
        assert_eq!(res_iter.next().unwrap(), &gt);
    }
}
