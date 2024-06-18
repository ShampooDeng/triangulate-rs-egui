use egui::Pos2;
use std::cmp::Ordering;

pub enum Orientation {
    ClockWise,
    CounterClockWise,
    Colinear,
}

pub enum MiddleVertexStatus {
    Convex,       // middle vertex is the heighest
    Concave,      // middle veretx is the lowest
    GradientUp,   // the y coordinates of vertex gradually increase
    GradientDown, // the y coordinates of vertex gradually decrease
}

/// Assessing 3 adjacent vertices' orientation by comparing their slopes.
pub fn cmp_slope(p: &Pos2, q: &Pos2, r: &Pos2) -> Orientation {
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
pub fn cmp_vertex_height(p: &Pos2, q: &Pos2, r: &Pos2) -> MiddleVertexStatus {
    match (q.y.ge(&p.y), q.y.le(&r.y)) {
        (true, false) => MiddleVertexStatus::Convex,
        (false, true) => MiddleVertexStatus::Concave,
        (true, true) => MiddleVertexStatus::GradientUp,
        (false, false) => MiddleVertexStatus::GradientDown,
    }
}
