use egui::Pos2;
use log::debug;
use std::{cmp::Ordering, collections::BTreeMap};

use crate::{dcel::DCEL, NIL};

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

struct Status {
    halfedge_idx: usize,
    helper: usize,
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

pub fn make_monotone(polygon: &DCEL) {
    let mut event_queue = polygon.event_queue();
    let mut btm: BTreeMap<i32, Status> = BTreeMap::new();

    while event_queue.len() != 0 {
        let idx = event_queue.pop().unwrap();
        match check_vertex_type(idx, &polygon) {
            VertexType::StartVertex => {
                // handle_start_vertex(idx, polygon, &mut btm);
                debug!("{} is start vertex", idx);
            }
            VertexType::EndVertex => {
                debug!("{} is end vertex", idx);
            }
            VertexType::RegularVetex => {
                debug!("{} is regular vertex", idx);
            }
            VertexType::SplitVertex => {
                debug!("{} is split vertex", idx);
            }
            VertexType::MergeVertex => {
                debug!("{} is merge vertex", idx);
            }
        }
        // WARN: this unfinished
    }
}

fn check_vertex_type(vertex_idx: usize, polygon: &DCEL) -> VertexType {
    let pre_idx;
    let next_idx;
    if vertex_idx == 0 {
        pre_idx = polygon.vertices.len() - 1;
        next_idx = vertex_idx + 1;
    } else if vertex_idx == polygon.vertices.len() - 1 {
        pre_idx = vertex_idx - 1;
        next_idx = 0;
    } else {
        pre_idx = vertex_idx - 1;
        next_idx = vertex_idx + 1;
    }

    let cur = polygon.vertices[vertex_idx].coordinates;
    let pre = polygon.vertices[pre_idx].coordinates;
    let next = polygon.vertices[next_idx].coordinates;
    let orientation = cmp_slope(pre, cur, next);
    let middle_vertex_status = cmp_vertex_height(pre, cur, next);
    match (orientation, middle_vertex_status) {
        (CounterClockWise, Convex) => VertexType::StartVertex,
        (CounterClockWise, Concave) => VertexType::EndVertex,
        (ClockWise, Convex) => VertexType::SplitVertex,
        (ClockWise, Concave) => VertexType::MergeVertex,
        (_, _) => VertexType::RegularVetex,
    }
}

fn handle_start_vertex(vertex_idx: usize, polygon: &DCEL, status: &mut BTreeMap<i32, Status>) {
    let edge_idx = polygon.vertices[vertex_idx].incident_edge;
    let origin_idx = polygon.halfedges[edge_idx].origin;
    let incident_edge = Status {
        halfedge_idx: edge_idx,
        helper: NIL,
    };
    status.insert(
        (polygon.vertices[origin_idx].coordinates.x * 100.).round() as i32,
        incident_edge,
    );
    debug!(
        "inserted edge's origin.x {}",
        polygon.vertices[origin_idx].coordinates.x
    );
}

fn handle_end_vertex() {
    todo!()
}

fn handle_split_vertex() {
    todo!()
}

fn handle_merge_vertex() {
    todo!()
}

fn handle_regular_vertex() {
    todo!()
}

#[cfg(test)]
mod tests {
    use crate::triangulate::{cmp_slope, cmp_vertex_height, MiddleVertexStatus, Orientation};
    use egui::Pos2;
    use std::collections::{BTreeMap, BTreeSet};
    struct Edge(f32, String);
    #[test]
    fn test_binary_tree_set() {
        let mut bts = BTreeSet::new();
        bts.insert(1);
        bts.insert(0);
        assert_eq!(bts.first(), Some(&0));
    }

    #[test]
    fn test_binary_tree_map() {
        let mut btm = BTreeMap::new();
        let edge1 = Edge(1., "hahah".to_string());
        let edge2 = Edge(5., "nice".to_string());
        let edge3 = Edge(2., "nice".to_string());
        let edge4 = Edge(4., "nice".to_string());
        let edge5 = Edge(1.5, "nice".to_string());
        // TODO: impl a trait to simplify the insert process
        btm.insert((edge1.0 * 100.).round() as i32, edge1);
        btm.insert((edge2.0 * 100.).round() as i32, edge2);
        btm.insert((edge3.0 * 100.).round() as i32, edge3);
        btm.insert((edge4.0 * 100.).round() as i32, edge4);
        btm.insert((edge5.0 * 100.).round() as i32, edge5);
        let keys = btm.into_keys().collect::<Vec<i32>>();
        assert_eq!(keys, &[100, 150, 200, 400, 500]);
        let high_neighbor = keys.partition_point(|&x| x <= 200);
        let low_neighbor = keys.partition_point(|&x| x < 200);
        assert_eq!(high_neighbor, 3);
        assert_eq!(low_neighbor, 2);
    }

    #[test]
    fn test_cmp_slope() {
        match cmp_slope(Pos2::new(1., 3.), Pos2::new(2., 2.), Pos2::new(1., 1.)) {
            Orientation::ClockWise => (),
            _ => panic!("match clockwise fail"),
        }
        match cmp_slope(Pos2::new(1., 1.), Pos2::new(2., 2.), Pos2::new(1., 3.)) {
            Orientation::CounterClockWise => (),
            _ => panic!("match counter-clockwise fail"),
        }
        match cmp_slope(Pos2::new(1., 1.), Pos2::new(2., 2.), Pos2::new(3., 3.)) {
            Orientation::Colinear => (),
            _ => panic!("match colinear fail"),
        }
        match cmp_slope(Pos2::new(1., 3.), Pos2::new(3., 3.), Pos2::new(3., 1.)) {
            Orientation::ClockWise => (),
            _ => panic!("match clockwise fail"),
        }
    }

    #[test]
    fn test_cmp_vertex_height() {
        match cmp_vertex_height(Pos2::new(1., 1.), Pos2::new(2., 2.), Pos2::new(3., 3.)) {
            MiddleVertexStatus::Gradient => (),
            _ => panic!("match gradient tyep fail"),
        }
        match cmp_vertex_height(Pos2::new(1., 3.), Pos2::new(3., 5.), Pos2::new(3., 3.)) {
            MiddleVertexStatus::Convex => (),
            _ => panic!("match convex type fail"),
        }
        match cmp_vertex_height(Pos2::new(1., 3.), Pos2::new(3., 0.), Pos2::new(3., 3.)) {
            MiddleVertexStatus::Concave => (),
            _ => panic!("match concave fail"),
        }
    }
}
