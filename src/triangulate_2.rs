use eframe::glow::{VertexArray, TRANSFORM_FEEDBACK_OVERFLOW};
use egui::Pos2;
use log::debug;
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::rc::Rc;
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
    Convex,   // middle vertex is the heighest
    Concave,  // middle veretx is the lowest
    Gradient, // middle vertex rests between pre and next vertices in y axis
}
use MiddleVertexStatus::{Concave, Convex};

struct Partition_Vertex {
    // point: Rc<Pos2>,
    point: Pos2,
    diag_points: Vec<Rc<Pos2>>,
    // current_diag: usize,
    // diag_endpoint: usize,
}
impl Partition_Vertex {
    fn new(input: &Pos2) -> Self {
        Partition_Vertex {
            // point: Rc::new(input.clone()),
            point: input.clone(),
            diag_points: Vec::new(),
        }
    }

    fn insert_diagonal(&mut self, vec: Rc<Pos2>) {
        self.diag_points.push(vec);
    }
}
struct Partition_Polygon {
    vertices: Vec<Rc<Partition_Vertex>>,
}

impl Partition_Polygon {
    fn new() -> Self {
        Partition_Polygon {
            vertices: Vec::new(),
        }
    }

    fn next_vertex(&self, idx: usize) -> usize {
        (idx + 1) % self.vertices.len()
    }

    fn prev_vertex(&self, idx: usize) -> usize {
        if idx == 0 {
            return self.vertices.len() - 1;
        }
        idx - 1
    }

    // fn insert_diagonal(&mut self, idx1:usize, idx2:usize) {
    //     self.vertices[idx1].insert_diagonal(self.vertices[idx2].point);
    //     self.vertices[idx2].insert_diagonal(self.vertices[idx1].point);
    // }

    fn build_from_pts(&mut self, input: &Vec<Pos2>) {
        let input_iter = input.iter();
        for item in input_iter {
            self.vertices.push(Rc::new(Partition_Vertex::new(item)));
        }
    }
}

type SearchTree = BTreeMap<usize, usize>;

struct Tree {
    search_tree: BTreeMap<i32, (usize, usize)>,
}

impl Tree {
    fn new() -> Self{
        Tree{
            search_tree: BTreeMap::new()
        }
    }

    pub fn push(&mut self, edge_origin_idx:usize, helper_idx:usize, poly: &Partition_Polygon) {
        let key = (poly.vertices[edge_origin_idx].point.x * 100.) as i32;
        self.search_tree.insert(key, (edge_origin_idx, helper_idx));
    }

    pub fn find(&self, vertex: &Rc<Partition_Vertex>) -> i32{
        let keys = self.search_tree.clone().into_keys().collect::<Vec<i32>>();
        let vertex_pos_x = (vertex.point.x *100.) as i32;
        match keys.binary_search(&vertex_pos_x){
            Ok(a) => {
                keys[a]
            }
            _ => {
                panic!("can't find vertex in Tree")
            }
        }
    }

    pub fn erase() {
        todo!()
    }
}

fn cmp_slope(p: Pos2, q: Pos2, r: Pos2) -> Orientation {
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

fn cmp_vertex_height(p: Pos2, q: Pos2, r: Pos2) -> MiddleVertexStatus {
    match (q.y.ge(&p.y), q.y.le(&r.y)) {
        (true, false) => MiddleVertexStatus::Convex,
        (false, true) => MiddleVertexStatus::Concave,
        _ => MiddleVertexStatus::Gradient,
    }
}

fn monoton_vertex_type(poly: &Partition_Polygon, idx: usize) -> VertexType {
    let prev: usize = poly.prev_vertex(idx);
    let next: usize = poly.next_vertex(idx);
    let p = poly.vertices[prev].point;
    let q = poly.vertices[idx].point;
    let r = poly.vertices[next].point;
    match (cmp_slope(p, q, r), cmp_vertex_height(p, q, r)) {
        (CounterClockWise, Convex) => VertexType::StartVertex,
        (CounterClockWise, Concave) => VertexType::EndVertex,
        (ClockWise, Convex) => VertexType::SplitVertex,
        (ClockWise, Concave) => VertexType::MergeVertex,
        (_, _) => VertexType::RegularVetex,
    }
}

fn to_event_queue(input: &Vec<Rc<Partition_Vertex>>) -> Vec<usize> {
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

fn handle_start_vertex(vertex_idx: usize, tree: &mut SearchTree) {
    let edge_origin_idx = vertex_idx;
    let helper_idx = vertex_idx;
    tree.insert(edge_origin_idx, helper_idx);
}

fn handle_end_vertex(vertex_idx: usize, tree: &mut Tree, poly: &Partition_Polygon) {
    let prev = poly.prev_vertex(vertex_idx);
    let a = tree.find(&poly.vertices[prev]);
    let helper_prev = tree.search_tree[&a].1;
    match monoton_vertex_type(poly, helper_prev) {
        VertexType::MergeVertex => {
            todo!()
        }
        _ => {}
    }
}

fn handle_regular_vertex() {
    todo!()
}

fn handle_split_vertex() {
    todo!()
}

fn handle_merge_vertex() {
    todo!()
}

pub fn monoton_polyon_partition(vertices: &Vec<Pos2>) {
    let mut partition_poly = Partition_Polygon::new();
    // let vertices_rc = vertices.iter().map(|x| Rc::new(x.clone()));
    partition_poly.build_from_pts(vertices);
    let mut tree = SearchTree::new();

    let mut event_queue = to_event_queue(&partition_poly.vertices);
    while !event_queue.is_empty() {
        let event_idx = event_queue.pop().unwrap();
        match monoton_vertex_type(&partition_poly, event_idx) {
            VertexType::StartVertex => {
                debug!("vertex{} is start vertex", event_idx);
                handle_start_vertex(event_idx, &mut tree);
            }
            VertexType::EndVertex => {
                debug!("vertex{} is end vertex", event_idx);
                // handle_end_vertex();
            }
            VertexType::RegularVetex => {
                debug!("vertex{} is regular vertex", event_idx);
                handle_regular_vertex();
            }
            VertexType::SplitVertex => {
                debug!("vertex{} is split vertex", event_idx);
                handle_split_vertex();
            }
            VertexType::MergeVertex => {
                debug!("vertex{} is merge vertex", event_idx);
                handle_merge_vertex();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{to_event_queue, Partition_Polygon, VertexType};
    use crate::triangulate_2::monoton_vertex_type;
    use egui::Pos2;

    #[test]
    fn test_build_from_pts() {
        let mut pts = Vec::new();
        pts.push(Pos2::new(1., 0.));
        pts.push(Pos2::new(2., 1.));
        pts.push(Pos2::new(2., 2.));
        pts.push(Pos2::new(0., 1.));

        let mut poly = Partition_Polygon::new();
        poly.build_from_pts(&pts);
        let mut gt_iter = pts.iter();
        for item in poly.vertices.iter() {
            assert_eq!(item.point, *gt_iter.next().unwrap())
        }
    }

    fn init_poly() -> Partition_Polygon {
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

        let mut poly = Partition_Polygon::new();
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
            .map(|x| vertex_type_to_string(x))
            .collect::<Vec<String>>();
        let gts = vec!["start", "start", "merge", "regular", "split", "end", "end"];
        assert_eq!(results_string, gts);
    }
}
