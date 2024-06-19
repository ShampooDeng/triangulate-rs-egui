use egui::Pos2;
use std::cmp::Ordering;
use std::f32::consts::PI;

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


pub fn vector_length(vector: (f32, f32)) -> f32 {
    (vector.0.powi(2) + vector.1.powi(2)).sqrt()
}

/// Compute the angle between vector1(cur -> next)
/// and vector2 (cur -> target).\
/// The angle is in 0 to 2pi, from vector1 to vector2.\
/// cur: current vertex\
/// next: cur's next vertex in partition polygon (in ccw order)
pub fn compute_angle(cur: &Pos2, next: &Pos2, target: &Pos2) -> f32 {
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
        panic!("Bro, what can i say")
    }
}