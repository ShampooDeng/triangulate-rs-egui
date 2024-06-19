use crate::monotone_y_partition::{monotone_partition, PartitionPolygon};
use crate::triangle_base::*;
use egui::Pos2;
use log::{debug, info};
use std::cmp::Ordering;
use std::fmt::Debug;

pub enum WhichSide {
    Left,
    Right,
}

impl Debug for WhichSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let WhichSide::Left = self {
            write!(f, "left")
        } else {
            write!(f, "right")
        }
    }
}

/// Check which side of the monotone polygon does a vertex belong to.
fn which_side(idx: usize, top_vertex_idx: usize, bottom_vertex_idx: usize) -> WhichSide {
    /* For monotone polygon defined in ccw order, the top and bottom vertex is
    vertex index: 0, 1, 2, ..., 8, 9.
                     ^          ^
    either           top        bottom
    or               bottom     top
    */
    if idx == top_vertex_idx || idx == bottom_vertex_idx {
        return WhichSide::Right;
    }

    // If the index of top is less than bottom,
    // then the interval of top and bottom is on the left side of the polygon.
    let between_top_and_bottom_is_left = if top_vertex_idx.lt(&bottom_vertex_idx) {
        true
    } else {
        false
    };
    let between_top_and_bottom = if (idx.lt(&top_vertex_idx) && idx.lt(&bottom_vertex_idx))
        || (idx.gt(&top_vertex_idx) && idx.gt(&bottom_vertex_idx))
    {
        false
    } else {
        true
    };

    match (between_top_and_bottom, between_top_and_bottom_is_left) {
        (true, true) | (false, false) => WhichSide::Left,
        (true, false) | (false, true) => WhichSide::Right,
    }
}

/// Check if two vertex is on the same of a monotone polygon
fn on_same_side(
    idx1: usize,
    idx2: usize,
    top_vertex_idx: usize,
    bottom_vertex_idx: usize,
) -> Option<WhichSide> {
    let result1 = which_side(idx1, top_vertex_idx, bottom_vertex_idx);
    let result2 = which_side(idx2, top_vertex_idx, bottom_vertex_idx);
    match (result1, result2) {
        (WhichSide::Left, WhichSide::Left) => Some(WhichSide::Left),
        (WhichSide::Right, WhichSide::Right) => Some(WhichSide::Right),
        _ => None,
    }
}

/// Check if a diagonal inside a monotone polygon
/// by assessing the orientation of event, last,
/// lastlast vertex (in process stack)'s orientation.
fn inside_mono_poly(
    cur: usize,
    last: usize,
    lastlast: usize,
    side: &WhichSide,
    vertices: &Vec<Pos2>,
) -> bool {
    let orientation = cmp_slope(&vertices[cur], &vertices[last], &vertices[lastlast]);
    match (side, orientation) {
        (WhichSide::Left, Orientation::ClockWise) => true,
        (WhichSide::Right, Orientation::CounterClockWise) => true,
        _ => false,
    }
}

/// Triangulate monotone polygon by
/// add new diagonals in PartitionPolygon
fn triangulate_monotone(
    partition_poly: &mut PartitionPolygon,
    monotone_poly: &[usize],
    vertices: &Vec<Pos2>,
) {
    // Partition is already a triangle
    if monotone_poly.len() <= 3 {
        return;
    }

    // Sort monotone partition's vertices by their coordinates.
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
    let top_vertex = event_stack.pop().unwrap(); // top vertex of monotone polygon
    let bottom_vertex = event_stack.first().unwrap().clone(); // bottom vertex of monotone polygon
    let mut prev_event_vertex = event_stack.pop().unwrap();
    process_stack.push(top_vertex); // push last vertex in event stack
    process_stack.push(prev_event_vertex); // push lastlast vertex in event stack

    while let Some(event_vertex) = event_stack.pop() {
        debug!("processing event vertex{}", event_vertex);
        debug!("process stack: {:?}", process_stack);
        debug!("event stack: {:?}", event_stack);

        if let Some(side) = on_same_side(
            event_vertex,
            *process_stack.last().unwrap(),
            top_vertex,
            bottom_vertex,
        ) {
            debug!(
                "vertex {},{} on {:?} side",
                event_vertex,
                process_stack.last().unwrap(),
                side
            );
            // Pop last in process stack, and add diagonal to all vertice
            // in process stack if possible. Stop until diagonal intersect
            // with polygon outlines, and push lastlast into process stack.
            let mut last = process_stack.pop().unwrap();
            while let Some(lastlast) = process_stack.pop() {
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
            // Pop all vertices in process stack, and insert diagonals between
            // current event vertex and them.
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

    // Insert diagonals between the bottom event vertex and 
    // all vertices left in process stack.
    let event_stack_bottom = event_stack.pop().unwrap();
    if process_stack.len() > 2 {
        for idx in process_stack[1..process_stack.len() - 1].iter() {
            partition_poly.insert_diagonal(event_stack_bottom, *idx);
        }
    }
}

/// Triangulate all monotone polygon partititons
pub fn polygon_triangulation(vertices: &Vec<Pos2>, mut partition_poly: &mut PartitionPolygon) -> Vec<Vec<Pos2>> {
    // let mut partition_poly = PartitionPolygon::new();
    partition_poly.build_from_pts(vertices);

    info!("---start monotone partition---");
    monotone_partition(&mut partition_poly);
    partition_poly.sort_diagonals(vertices);
    let mut monotone_polygons: Vec<Vec<usize>> = Vec::new();
    partition_poly.make_polygons(0, &mut monotone_polygons, vertices);
    partition_poly.reset_unused_diag_counts();

    info!("---start triangulate monotone polygon---");
    while let Some(monotone_poly) = monotone_polygons.pop() {
        info!("processing mono polygon: {:?}", monotone_poly);
        triangulate_monotone(&mut partition_poly, &monotone_poly, vertices);
    }

    // Generate monotone polygon partition and output coordinates
    partition_poly.partition(vertices)
}
