use core::panic;
use egui::Pos2;
use log::debug;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::f32::consts::PI;
pub enum VertexType {
    StartVertex,
    EndVertex,
    RegularVetex,
    SplitVertex,
    MergeVertex,
}

pub enum Orientation {
    ClockWise,
    CounterClockWise,
    Colinear,
}
use Orientation::{ClockWise, CounterClockWise};

pub enum MiddleVertexStatus {
    Convex,       // middle vertex is the heighest
    Concave,      // middle veretx is the lowest
    GradientUp,   // the y coordinates of vertex gradually increase
    GradientDown, // the y coordinates of vertex gradually decrease
}
use MiddleVertexStatus::{Concave, Convex};

type Vertices = Vec<Pos2>;

struct PartitionVertex {
    point: Pos2,
    diag_points: Vec<usize>,
    current: usize,
}

impl PartitionVertex {
    fn new(input: &Pos2) -> Self {
        PartitionVertex {
            point: *input, // Pos2 has copy trait, so just dereference it.
            // NOTE: Vec<T> can't be deep copied
            diag_points: Vec::new(),
            current: 0,
        }
    }

    fn insert_diagnoal(&mut self, vertex_idx: usize) {
        self.diag_points.push(vertex_idx);
    }

    fn magnified_pos_x(&self) -> i32 {
        (self.point.x * 100.).round() as i32
    }

    /// Sort diagnoals by their agnle relative to the line\
    /// formed by self and its' next vertex in partition polygon in ccw.
    fn sort_diagnoals(&mut self, next: &Pos2, vertices: &Vertices) {
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

    fn diag_is_end(&self) -> bool {
        if self.current == self.diag_points.len() {
            return true;
        }
        false
    }

    fn current_diag(&self) -> usize {
        self.diag_points[self.current]
    }

    fn has_unused_diag(&self) -> bool {
        if self.diag_points.is_empty() {
            return false;
        }
        true
    }
}

fn vector_length(vector: (f32, f32)) -> f32 {
    (vector.0.powi(2) + vector.1.powi(2)).sqrt()
}

/// Compute the angle between vector1(cur -> next)
/// and vector2 (cur -> target).\
/// The angle is in 0 to 2pi, from vector1 to vector2.\
/// cur: current vertex\
/// next: cur's next vertex in partition polygon
fn compute_angle(cur: &Pos2, next: &Pos2, target: &Pos2) -> f32 {
    let ref_vector = (next.x - cur.x, next.y - cur.y);
    let target_vector = (target.x - cur.x, target.y - cur.y);
    let cos = (ref_vector.0 * target_vector.0 + ref_vector.1 * target_vector.1)
        / (vector_length(ref_vector) * vector_length(target_vector));
    let sin = (ref_vector.0 * target_vector.1 - ref_vector.1 * target_vector.0)
        / (vector_length(ref_vector) * vector_length(target_vector));
    if let Some(res) = sin.partial_cmp(&0.0) {
        match res {
            Ordering::Less => 2. * PI - cos.acos(),
            _ => cos.acos(),
        }
    } else {
        panic!("what can i say")
    }
}

struct PartitionPolygon {
    vertices: Vec<PartitionVertex>,
}

impl PartitionPolygon {
    fn new() -> Self {
        PartitionPolygon {
            vertices: Vec::new(),
        }
    }

    /// Get next vertex's index
    fn next_vertex(&self, idx: usize) -> usize {
        (idx + 1) % self.vertices.len()
    }

    /// Get previous vertex's index
    fn prev_vertex(&self, idx: usize) -> usize {
        if idx == 0 {
            return self.vertices.len() - 1;
        }
        idx - 1
    }

    /// Insert diagnoal between two vertices given
    /// their index in partition polygon
    fn insert_diagnoal(&mut self, idx1: usize, idx2: usize) {
        self.vertices[idx1].insert_diagnoal(idx2);
        self.vertices[idx2].insert_diagnoal(idx1);
    }

    /// Build a partition polygon from a list of vertices
    /// in ccw order.
    fn build_from_pts(&mut self, input: &[Pos2]) {
        let input_iter = input.iter();
        for item in input_iter {
            self.vertices.push(PartitionVertex::new(item));
        }
    }

    fn partition(&mut self, vertices: &Vertices) -> Vec<Vec<Pos2>> {
        // HACK: change sort_diagnoals's arg from PartitionVertex to Pos2,
        // since Vec<T> in PartitionVertex will cause multiple mutable
        // borrow of self, here, in this function.
        for idx in 0..self.vertices.len() {
            if !self.vertices[idx].diag_points.is_empty() {
                let next_pos = self.vertices[(idx + 1) % self.vertices.len()].point;
                let cur = &mut self.vertices[idx];
                cur.sort_diagnoals(&next_pos, vertices);
            }
        }
        let mut result: Vec<Vec<Pos2>> = Vec::new();
        self.make_polygons(0, &mut result);
        result
    }

    fn make_polygons(&mut self, start: usize, result: &mut Vec<Vec<Pos2>>) -> usize {
        let mut new_polygon: Vec<Pos2> = Vec::new();
        let mut idx: usize = start;
        loop {
            new_polygon.push(self.vertices[idx].point);
            if self.vertices[idx].has_unused_diag() {
                let diag = self.vertices[idx].diag_points.pop().unwrap();
                if diag != start {
                    idx = self.make_polygons(idx, result);
                } else {
                    result.push(new_polygon);
                    return idx;
                }
            } else {
                idx = (idx + 1) % self.vertices.len();
            }

            if idx == start {
                result.push(new_polygon);
                break;
            }
        }
        idx
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
            keys: Vec::new(),
        }
    }

    fn update_keys(&mut self) {
        self.keys = self.search_tree.clone().into_keys().collect::<Vec<i32>>();
    }

    pub fn insert(&mut self, edge_origin_idx: usize, helper_idx: usize, poly: &PartitionPolygon) {
        let key = poly.vertices[edge_origin_idx].magnified_pos_x();
        self.search_tree
            .insert(key, PartitionTreeEntry::new(edge_origin_idx, helper_idx));
        self.update_keys();
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
        if self.search_tree.remove(&entry_key).is_none() {
            return Err(entry_key);
        }
        self.update_keys();
        Ok(())
    }

    /// Find the a vertex's nearest neighbor in tree
    pub fn lower_bound(&self, vertex: &PartitionVertex) -> i32 {
        // HACK: return 0 when search tree is empty
        // Will this cause any bug?
        if self.search_tree.is_empty() {
            return 0;
        }
        let pred = vertex.magnified_pos_x();
        let low = self.keys.partition_point(|x| x < &pred) - 1;
        self.keys[low]
    }
}

/// Get event vertex's left neighbor in the search tree
fn get_left_neighbor(vertex: &PartitionVertex, tree: &PartitionTree) -> (i32, usize) {
    let key = tree.lower_bound(vertex);
    (key, tree.search_tree[&key].helper)
}

fn get_left_neighbor_helper(vertex: &PartitionVertex, tree: &PartitionTree) -> usize {
    let key = tree.lower_bound(vertex);
    tree.search_tree[&key].helper
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

/// Assessing 3 adjacent vertices' orientation by comparing
/// their slopes
fn cmp_slope(p: &Pos2, q: &Pos2, r: &Pos2) -> Orientation {
    let slope_pq = (q.y - p.y) * (r.x - p.x);
    let slope_pr = (r.y - p.y) * (q.x - p.x);
    match slope_pq.partial_cmp(&slope_pr) {
        Some(Ordering::Equal) => Orientation::Colinear,
        // (y_q-y_p)(x_r-x_p) > (y_r-y_p)(x_q-x_p) => cw
        Some(Ordering::Greater) => Orientation::ClockWise,
        // (y_q-y_p)(x_r-x_p) < (y_r-y_p)(x_q-x_p) => ccw
        Some(Ordering::Less) => Orientation::CounterClockWise,
        None => panic!(
            "Comparison between {}, {} is impossible",
            slope_pq, slope_pr
        ),
    }
}

/// Compare vertex's height with its left and right neighbors
fn cmp_vertex_height(p: &Pos2, q: &Pos2, r: &Pos2) -> MiddleVertexStatus {
    match (q.y.ge(&p.y), q.y.le(&r.y)) {
        (true, false) => MiddleVertexStatus::Convex,
        (false, true) => MiddleVertexStatus::Concave,
        (true, true) => MiddleVertexStatus::GradientUp,
        (false, false) => MiddleVertexStatus::GradientDown,
    }
}

/// Check a vertex's type by assessing their orientation
/// (ccw, cw) and its position with respect to its two neighbor vertices.
fn monoton_vertex_type(poly: &PartitionPolygon, idx: usize) -> VertexType {
    let prev: usize = poly.prev_vertex(idx);
    let next: usize = poly.next_vertex(idx);
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

/// Generate event queue of given vertices.\
/// All vertices are sorted by their y coordinates (from top to bottom).\
/// If vertices are at the same height, they will
/// be sorted by x coordinates (from left to right).
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
    let prev = poly.prev_vertex(vertex_idx);
    let search_key = tree.find(&poly.vertices[prev]);
    let helper_prev_idx = tree.search_tree[&search_key].helper;
    if let VertexType::MergeVertex = monoton_vertex_type(poly, helper_prev_idx) {
        poly.insert_diagnoal(vertex_idx, helper_prev_idx);
    }
    let _ = tree.erase(search_key);
}

/// Check if the polygon interior is in the right of regular vertex.\
/// Assuming all vertices are sorted in CCW order.
fn polygon_interior_to_right(vertex_idx: usize, poly: &PartitionPolygon) -> Result<bool, ()> {
    let prev: usize = poly.prev_vertex(vertex_idx);
    let next: usize = poly.next_vertex(vertex_idx);
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
    let interior_to_right =  match polygon_interior_to_right(vertex_idx, poly) {
        Ok(res) => res,
        Err(_) => panic!("wrong middlevertex status"),
    };
    if interior_to_right {
        debug!("interior is right to vertex{}", vertex_idx);
        let prev = poly.prev_vertex(vertex_idx);
        let search_key = tree.find(&poly.vertices[prev]);
        let helper_prev_idx = tree.search_tree[&search_key].helper;
        if let VertexType::MergeVertex = monoton_vertex_type(poly, helper_prev_idx) {
            poly.insert_diagnoal(vertex_idx, helper_prev_idx);
        }
        let _ = tree.erase(search_key);
        tree.insert(vertex_idx, vertex_idx, poly);
    } else {
        debug!("interior is left to vertex{}", vertex_idx);
        let (left_neighbor_edge_key, left_neigbor_edge_helper) =
            get_left_neighbor(&poly.vertices[vertex_idx], tree);
        if let VertexType::MergeVertex = monoton_vertex_type(poly, left_neigbor_edge_helper) {
            poly.insert_diagnoal(vertex_idx, left_neigbor_edge_helper);
        }
        update_helper(left_neighbor_edge_key, vertex_idx, tree);
    }
}

fn handle_split_vertex(vertex_idx: usize, tree: &mut PartitionTree, poly: &mut PartitionPolygon) {
    let (left_neighbor_edge_key, left_neigbor_edge_helper) =
        get_left_neighbor(&poly.vertices[vertex_idx], tree);
    poly.insert_diagnoal(vertex_idx, left_neigbor_edge_helper);
    update_helper(left_neighbor_edge_key, vertex_idx, tree);
    tree.insert(vertex_idx, vertex_idx, poly);
}

fn handle_merge_vertex(vertex_idx: usize, tree: &mut PartitionTree, poly: &mut PartitionPolygon) {
    let prev = poly.prev_vertex(vertex_idx);
    let search_key = tree.find(&poly.vertices[prev]);
    let helper_prev_idx = tree.search_tree[&search_key].helper;
    if let VertexType::MergeVertex = monoton_vertex_type(poly, helper_prev_idx) {
        poly.insert_diagnoal(vertex_idx, helper_prev_idx);
    }
    let _ = tree.erase(search_key);
    let (left_neighbor_edge_key, left_neigbor_edge_helper) =
        get_left_neighbor(&poly.vertices[vertex_idx], tree);
    if let VertexType::MergeVertex = monoton_vertex_type(poly, left_neigbor_edge_helper) {
        poly.insert_diagnoal(vertex_idx, left_neigbor_edge_helper);
    }
    update_helper(left_neighbor_edge_key, vertex_idx, tree);
}

pub fn monoton_polyon_partition(vertices: &Vec<Pos2>) -> Vec<Vec<Pos2>> {
    let mut partition_poly = PartitionPolygon::new();
    // let vertices_rc = vertices.iter().map(|x| Rc::new(x.clone()));
    partition_poly.build_from_pts(vertices);
    let mut tree = PartitionTree::new();

    let mut event_queue = to_event_queue(&partition_poly.vertices);
    while let Some(event_idx) = event_queue.pop() {
        match monoton_vertex_type(&partition_poly, event_idx) {
            VertexType::StartVertex => {
                debug!("vertex{} is start vertex", event_idx);
                handle_start_vertex(event_idx, &mut tree, &partition_poly);
            }
            VertexType::EndVertex => {
                debug!("vertex{} is end vertex", event_idx);
                handle_end_vertex(event_idx, &mut tree, &mut partition_poly);
            }
            VertexType::RegularVetex => {
                debug!("vertex{} is regular vertex", event_idx);
                handle_regular_vertex(event_idx, &mut tree, &mut partition_poly);
            }
            VertexType::SplitVertex => {
                debug!("vertex{} is split vertex", event_idx);
                handle_split_vertex(event_idx, &mut tree, &mut partition_poly);
            }
            VertexType::MergeVertex => {
                debug!("vertex{} is merge vertex", event_idx);
                handle_merge_vertex(event_idx, &mut tree, &mut partition_poly);
            }
        }
    }
    // Debug only
    // diagnoals: 5<->3, 1<->3
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
        monoton_polyon_partition, to_event_queue, PartitionPolygon, PartitionTree, PartitionVertex,
        VertexType,
    };
    use crate::monotone_y_partition::monoton_vertex_type;
    use egui::Pos2;

    #[test]
    fn test_build_from_pts() {
        let mut pts = Vec::new();
        pts.push(Pos2::new(1., 0.));
        pts.push(Pos2::new(2., 1.));
        pts.push(Pos2::new(2., 2.));
        pts.push(Pos2::new(0., 1.));

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
        let mut pts = Vec::new();
        pts.push(Pos2::new(1., 0.));
        pts.push(Pos2::new(2., 1.));
        pts.push(Pos2::new(3., 0.));
        pts.push(Pos2::new(5., 1.5));
        pts.push(Pos2::new(3.5, 3.));
        pts.push(Pos2::new(1.5, 1.5));
        pts.push(Pos2::new(1., 2.4));

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
        assert_eq!(tree.lower_bound(&poly.vertices[1]), 150);
    }

    #[test]
    fn test_sort_diag() {
        let mut partition_vertex = PartitionVertex::new(&Pos2::new(10., 10.));
        partition_vertex.diag_points = (0..6).collect();
        let next = Pos2::new(12., 8.);
        let mut vertices = Vec::new();
        vertices.push(Pos2::new(6., 7.));
        vertices.push(Pos2::new(4., 15.));
        vertices.push(Pos2::new(2., 10.));
        vertices.push(Pos2::new(10., 20.));
        vertices.push(Pos2::new(15., 10.));
        vertices.push(Pos2::new(8., 18.));
        partition_vertex.sort_diagnoals(&next, &vertices);
        let res = partition_vertex.diag_points;
        let gts = vec![4, 3, 5, 1, 2, 0];
        assert_eq!(res, gts);
    }

    #[test]
    fn test_monotone_partition() {
        let mut pts = Vec::new();
        pts.push(Pos2::new(157., 29.)); // 0
        pts.push(Pos2::new(308., 173.)); // 1
        pts.push(Pos2::new(481., 49.)); // 2
        pts.push(Pos2::new(624., 180.)); // 3
        pts.push(Pos2::new(500., 349.)); // 4
        pts.push(Pos2::new(378., 286.)); // 5
        pts.push(Pos2::new(185., 333.)); // 6
        let result = monoton_polyon_partition(&pts);
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
