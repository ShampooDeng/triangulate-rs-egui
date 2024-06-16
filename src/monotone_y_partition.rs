use core::panic;
use egui::Pos2;
use log::{debug, info};
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

pub enum WhichSide {
    Left,
    Right,
}

// HACK: A copy of PartitionPolygon vertices,
// used as auxilary reference in sort_daig() and partition()
type Vertices = Vec<Pos2>;

pub trait Circulator {
    fn prev(&self, idx: usize) -> usize;
    fn next(&self, idx: usize) -> usize;
}

impl Circulator for Vec<usize> {
    fn prev(&self, idx: usize) -> usize {
        (idx + 1) % self.len()
    }

    fn next(&self, idx: usize) -> usize {
        if idx == 0 {
            return self.len() - 1;
        }
        idx - 1
    }
}

impl Circulator for [usize] {
    fn prev(&self, idx: usize) -> usize {
        (idx + 1) % self.len()
    }

    fn next(&self, idx: usize) -> usize {
        if idx == 0 {
            return self.len() - 1;
        }
        idx - 1
    }
}

struct PartitionVertex {
    point: Pos2,
    diag_points: Vec<usize>,
    unused_diag_count: usize,
}

impl PartitionVertex {
    fn new(input: &Pos2) -> Self {
        PartitionVertex {
            point: *input, // Pos2 has copy trait, so just dereference it.
            // NOTE: Vec<T> can't be deep copied
            diag_points: Vec::new(),
            unused_diag_count: 0,
        }
    }

    fn insert_diagonal(&mut self, vertex_idx: usize) {
        if let Some(_) = self.diag_points.iter().find(|&&x| x==vertex_idx) {
            return;
        }
        self.diag_points.push(vertex_idx);
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

pub struct PartitionPolygon {
    vertices: Vec<PartitionVertex>,
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
    fn new() -> Self {
        PartitionPolygon {
            vertices: Vec::new(),
        }
    }

    /// Insert diagonal between two vertices given
    /// their index in partition polygon
    fn insert_diagonal(&mut self, idx1: usize, idx2: usize) {
        if self.next(idx1) == idx2 || self.prev(idx1) == idx2 {
            return;
        }
        info!("insert diagonal between {} and {}", idx1, idx2);
        self.vertices[idx1].insert_diagonal(idx2);
        self.vertices[idx2].insert_diagonal(idx1);
    }

    /// Build a partition polygon from a list of vertices
    /// in ccw order.
    fn build_from_pts(&mut self, input: &[Pos2]) {
        let input_iter = input.iter();
        for item in input_iter {
            self.vertices.push(PartitionVertex::new(item));
        }
    }

    fn reset_unused_diag_counts(&mut self) {
        info!("reset polygon vertices' diag count");
        for vertex in self.vertices.iter_mut() {
            vertex.reset_unused_diag_count();
        }
    }

    /// Use this method with rest_unused_diag_count()
    fn make_polygons(&mut self, start: usize, result: &mut Vec<Vec<usize>>) -> usize {
        // TODO: description for this function
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
                    result.push(new_polygon);
                    return idx;
                } else {
                    idx = self.make_polygons(idx, result);
                }
            } else {
                idx = (idx + 1) % self.vertices.len();
            }

            if idx == start && new_polygon.len() > 2 {
                debug!("push into result:{:?}", new_polygon);
                result.push(new_polygon);
                break;
            }
        }
        idx
    }

    fn output_coordinates(&mut self, result: &Vec<Vec<usize>>) -> Vec<Vec<Pos2>> {
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

    fn sort_diagonals(&mut self, vertices: &Vertices) {
        info!("sort polygon vertices' diag");
        for idx in 0..self.vertices.len() {
            if !self.vertices[idx].diag_points.is_empty() {
                let next_pos = self.vertices[(idx + 1) % self.vertices.len()].point;
                self.vertices[idx].sort_diag(&next_pos, vertices);
                // cur.sort_diag(&next_pos, vertices);
            }
            debug!("vertex{}'s diag: {:?}", idx, self.vertices[idx].diag_points);
        }
    }

    /// Output partitions described by a vector of vertices' coordinates
    fn partition(&mut self, vertices: &Vertices) -> Vec<Vec<Pos2>> {
        // HACK: change sort_diagonals's arg from PartitionVertex to Pos2,
        // since Vec<T> in PartitionVertex will cause multiple mutable
        // borrow of self, here, in this function.
        self.sort_diagonals(vertices);
        let mut result: Vec<Vec<usize>> = Vec::new();
        info!("---start making polygons---");
        let ret = self.make_polygons(0, &mut result);
        debug!("the return of make_polygons:{}", ret);
        self.reset_unused_diag_counts();
        debug!("the num of polygon partition:{}", result.len());
        self.output_coordinates(&result)
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
        debug!("tree search key before search: {:?}", tree.search_tree.keys());
        // BUG: If a regular vertex is colinear to a neighbor end vertex,
        // then "no entry found for key" or "attempt to subtract with overflow in line 364"
        let (left_neighbor_edge_key, left_neigbor_edge_helper) =
            get_left_neighbor(&poly.vertices[vertex_idx], tree);
        if let VertexType::MergeVertex = monoton_vertex_type(poly, left_neigbor_edge_helper) {
            poly.insert_diagonal(vertex_idx, left_neigbor_edge_helper);
        }
        update_helper(left_neighbor_edge_key, vertex_idx, tree);
    }
}

fn handle_split_vertex(vertex_idx: usize, tree: &mut PartitionTree, poly: &mut PartitionPolygon) {
    let (left_neighbor_edge_key, left_neigbor_edge_helper) =
        get_left_neighbor(&poly.vertices[vertex_idx], tree);
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
        get_left_neighbor(&poly.vertices[vertex_idx], tree);
    if let VertexType::MergeVertex = monoton_vertex_type(poly, left_neigbor_edge_helper) {
        poly.insert_diagonal(vertex_idx, left_neigbor_edge_helper);
    }
    update_helper(left_neighbor_edge_key, vertex_idx, tree);
}

fn monotone_partition(mut partition_poly: &mut PartitionPolygon) {
    let mut tree = PartitionTree::new();
    let mut event_queue = to_event_queue(&partition_poly.vertices);
    debug!("monotone partition event queue:{:?}", event_queue);
    while let Some(event_idx) = event_queue.pop() {
        match monoton_vertex_type(&partition_poly, event_idx) {
            VertexType::StartVertex => {
                info!("vertex{} is start vertex", event_idx);
                handle_start_vertex(event_idx, &mut tree, &partition_poly);
            }
            VertexType::EndVertex => {
                info!("vertex{} is end vertex", event_idx);
                handle_end_vertex(event_idx, &mut tree, &mut partition_poly);
            }
            VertexType::RegularVetex => {
                info!("vertex{} is regular vertex", event_idx);
                handle_regular_vertex(event_idx, &mut tree, &mut partition_poly);
            }
            VertexType::SplitVertex => {
                info!("vertex{} is split vertex", event_idx);
                handle_split_vertex(event_idx, &mut tree, &mut partition_poly);
            }
            VertexType::MergeVertex => {
                info!("vertex{} is merge vertex", event_idx);
                handle_merge_vertex(event_idx, &mut tree, &mut partition_poly);
            }
        }
    }
}

pub fn monoton_polygon_partition(vertices: &Vec<Pos2>) -> Vec<Vec<Pos2>> {
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

/// Check if the Monotone polygon's interior is right to a vertex
fn mono_interior_to_right(
    idx: usize,
    top_vertex_idx: usize,
    vertices: &Vec<Pos2>,
) -> Result<bool, ()> {
    // let prev: usize = mono_poly.prev(idx);
    // let next: usize = mono_poly.next(idx);
    // let p = &vertices[prev];
    // let q = &vertices[idx];
    // let r = &vertices[next];
    // match cmp_vertex_height(p, q, r) {
    //     MiddleVertexStatus::GradientDown => Ok(true),
    //     MiddleVertexStatus::GradientUp => Ok(false),
    //     _ => Err(()),
    // }
    let mono_top_x = vertices[top_vertex_idx].x;
    let target_x = vertices[idx].x;
    let result = target_x.partial_cmp(&mono_top_x).unwrap();
    match result {
        Ordering::Less => Ok(true),
        Ordering::Greater => Ok(false),
        Ordering::Equal => Err(()),
    }
}

fn on_same_side(
    idx1: usize,
    idx2: usize,
    top_vertex_idx: usize,
    vertices: &Vec<Pos2>,
) -> Option<WhichSide> {
    let result1 = mono_interior_to_right(idx1, top_vertex_idx, vertices).unwrap_or_else(|_err| {
        panic!("processing vertex {idx1}");
    });
    let result2 = mono_interior_to_right(idx2, top_vertex_idx, vertices).unwrap_or_else(|_err| {
        panic!("processing vertex {idx2}");
    });
    match (result1, result2) {
        (true, true) => Some(WhichSide::Left),
        (false, false) => Some(WhichSide::Right),
        _ => None,
    }
}

// BUG: check this function
fn inside_mono_poly(
    cur: usize,
    last: usize,
    lastlast: usize,
    side: &WhichSide,
    vertices: &Vec<Pos2>,
) -> bool {
    let orientation: Orientation;
    if let WhichSide::Left = side {
        debug!("on left side");
        orientation = cmp_slope(&vertices[lastlast], &vertices[last], &vertices[cur]);
    } else {
        debug!("on right side");
        orientation = cmp_slope(&vertices[cur], &vertices[last], &vertices[lastlast]);
    }
    match (side, orientation) {
        (WhichSide::Left, CounterClockWise) => true,
        (WhichSide::Right, CounterClockWise) => true,
        _ => false,
    }
}

fn triangulate_monotone(
    partition_poly: &mut PartitionPolygon,
    monotone_poly: &[usize],
    vertices: &Vec<Pos2>,
) {
    if monotone_poly.len() <= 3 {
        return;
    }

    let mut event_stack: Vec<usize> = Vec::new();
    monotone_poly.clone_into(&mut event_stack);
    event_stack.sort_by(|a, b| {
        // HACK: can't index into muttable vector, must use vertices instead.
        let a_pos = vertices[*a];
        let b_pos = vertices[*b];
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
    debug!("event_stack after sort: {:?}", event_stack);

    let mut process_stack: Vec<usize> = Vec::new();
    let top_vertex = event_stack.pop().unwrap();
    process_stack.push(top_vertex); // push event_vertex1
    let mut prev_event_vertex = event_stack.pop().unwrap(); // save event_vertex2
    process_stack.push(prev_event_vertex); // push event_vertex2

    while let Some(event_vertex) = event_stack.pop() {
        debug!("processing event vertex{}", event_vertex);
        debug!("process stack: {:?}", process_stack);
        debug!("event stack: {:?}", event_stack);

        if let Some(side) = on_same_side(
            event_vertex,
            *process_stack.last().unwrap(),
            top_vertex,
            vertices,
        ) {
            debug!(
                "vertex {},{} on same side",
                event_vertex,
                process_stack.last().unwrap()
            );
            let mut last = process_stack.pop().unwrap();
            while let Some(lastlast) = process_stack.pop() {
                // let lastlast = *process_stack.last().unwrap();
                if inside_mono_poly(event_vertex, last, lastlast, &side, vertices) {
                    partition_poly.insert_diagonal(lastlast, event_vertex);
                    last = lastlast;
                } else {
                    process_stack.push(lastlast);
                    break;
                }
            }
            process_stack.push(last);
            process_stack.push(event_vertex);
        } else {
            while let Some(vertex_idx) = process_stack.pop() {
                partition_poly.insert_diagonal(vertex_idx, event_vertex);
            }
            process_stack.push(prev_event_vertex);
            process_stack.push(event_vertex);
        }

        if event_stack.len() == 1 {
            break;
        }
        prev_event_vertex = event_vertex;
    }

    let event_stack_bottom = event_stack.pop().unwrap();
    if process_stack.len() > 2 {
        for idx in process_stack[1..process_stack.len() - 1].iter() {
            partition_poly.insert_diagonal(event_stack_bottom, *idx);
        }
    }
}

pub fn polygon_triangulation(vertices: &Vec<Pos2>) -> Vec<Vec<Pos2>> {
    let mut partition_poly = PartitionPolygon::new();
    partition_poly.build_from_pts(vertices);

    info!("---start monotone partition---");
    monotone_partition(&mut partition_poly);
    partition_poly.sort_diagonals(vertices);
    let mut monotone_polygons: Vec<Vec<usize>> = Vec::new();
    partition_poly.make_polygons(0, &mut monotone_polygons);
    partition_poly.reset_unused_diag_counts();
    info!("---start triangulate monotone polygon---");
    while let Some(monotone_poly) = monotone_polygons.pop() {
        info!("processing mono polygon: {:?}", monotone_poly);
        triangulate_monotone(&mut partition_poly, &monotone_poly, vertices);
    }
    partition_poly.partition(vertices)
}

#[cfg(test)]
mod tests {
    use std::vec;

    use super::{
        monoton_polygon_partition, to_event_queue, PartitionPolygon, PartitionTree,
        PartitionVertex, VertexType,
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
        partition_vertex.sort_diag(&next, &vertices);
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
        let result = monoton_polygon_partition(&pts);
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
