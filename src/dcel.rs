/// The following code is adapted from https://docs.rs/crate/voronoi/0.1.4/source/src/dcel.rs
use crate::NIL;
use egui::Pos2 as Point;
use log::info;
use std::{cmp::Ordering, fmt};

/// Doubly Connected Edge List representation of a subdivision of the plane.
pub struct DCEL {
    /// Vertices
    pub vertices: Vec<Vertex>,
    /// Halfedges
    pub halfedges: Vec<HalfEdge>,
    /// Faces
    pub faces: Vec<Face>,
}

impl DCEL {
    /// Construct an empty DCEL
    pub fn new() -> Self {
        DCEL {
            vertices: vec![],
            halfedges: vec![],
            faces: vec![],
        }
    }

    /// Add two halfedges that are twins
    pub fn add_twins(&mut self) -> (usize, usize) {
        let mut he1 = HalfEdge::new();
        let mut he2 = HalfEdge::new();

        let start_index = self.halfedges.len();
        he1.twin = start_index + 1;
        he2.twin = start_index;
        self.halfedges.push(he1);
        self.halfedges.push(he2);
        (start_index, start_index + 1)
    }

    /// Get the origin of a halfedge by index
    pub fn get_origin(&self, edge: usize) -> Point {
        let origin_ind = self.halfedges[edge].origin;
        return self.vertices[origin_ind].coordinates;
    }

    /// Set the previous edge of all halfedges
    /// Assumes that the DCEL is well-formed.
    pub fn set_prev(&mut self) {
        let mut seen_edges = vec![false; self.halfedges.len()];
        for edge_ind in 0..self.halfedges.len() {
            if seen_edges[edge_ind] {
                continue;
            }
            let mut current_ind = edge_ind;
            // BUG: the following line does nothing
            // seen_edges[current_ind];
            loop {
                let next_edge = self.halfedges[current_ind].next;
                self.halfedges[next_edge].prev = current_ind;
                current_ind = next_edge;
                seen_edges[current_ind] = true;
                if current_ind == edge_ind {
                    break;
                }
            }
        }
    }

    pub fn event_queue(&self) -> Vec<usize> {
        let mut event_queue = Vec::from_iter(0..self.vertices.len());
        event_queue.sort_by(|a, b| {
            let a_coordinate = self.vertices[*a].coordinates;
            let b_coordinate = self.vertices[*b].coordinates;
            let mut result = a_coordinate.y.partial_cmp(&b_coordinate.y).unwrap();
            if result.is_eq() {
                result = a_coordinate.x.partial_cmp(&b_coordinate.x).unwrap();
                match result {
                    Ordering::Less => result = Ordering::Greater,
                    Ordering::Greater => result = Ordering::Less,
                    _ => {}
                }
            }
            result
        });
        event_queue
    }

    //     fn remove_edge(&mut self, edge: usize) {
    //         let edge_prev = self.halfedges[edge].prev;
    //         let edge_next = self.halfedges[edge].next;
    //         let twin = self.halfedges[edge].twin;
    //         let twin_prev = self.halfedges[twin].prev;
    //         let twin_next = self.halfedges[twin].next;

    //         self.halfedges[edge_prev].next = twin_next;
    //         self.halfedges[edge_next].prev = twin_prev;
    //         self.halfedges[twin_prev].next = edge_next;
    //         self.halfedges[twin_next].prev = edge_prev;

    //         self.halfedges[edge].alive = false;
    //         self.halfedges[twin].alive = false;
    //     }

    //     fn get_edges_around_vertex(&self, vertex: usize) -> Vec<usize> {
    //         let mut result = vec![];
    //         let start_edge = self.vertices[vertex].incident_edge;
    //         let mut current_edge = start_edge;
    //         loop {
    //             result.push(current_edge);
    //             let current_twin = self.halfedges[current_edge].twin;
    //             current_edge = self.halfedges[current_twin].next;
    //             if current_edge == start_edge {
    //                 break;
    //             }
    //         }
    //         return result;
    //     }

    //     /// Remove a vertex and all attached halfedges.
    //     /// Does not affect faces!!
    //     pub fn remove_vertex(&mut self, vertex: usize) {
    //         let vertex_edges = self.get_edges_around_vertex(vertex);
    //         for edge in vertex_edges {
    //             self.remove_edge(edge);
    //         }
    //         self.vertices[vertex].alive = false;
    //     }
}

impl fmt::Debug for DCEL {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut vertices_disp = String::new();

        for (index, node) in self.vertices.iter().enumerate() {
            if node.alive {
                vertices_disp.push_str(format!("{}: {:?}\n", index, node).as_str());
            }
        }

        let mut faces_disp = String::new();

        for (index, node) in self.faces.iter().enumerate() {
            if node.alive {
                faces_disp.push_str(format!("{}: {}\n", index, node).as_str());
            }
        }

        let mut halfedges_disp = String::new();

        for (index, node) in self.halfedges.iter().enumerate() {
            if node.alive {
                halfedges_disp.push_str(format!("{}: {:?}\n", index, node).as_str());
            }
        }

        write!(
            f,
            "Vertices:\n{}\nFaces:\n{}\nHalfedges:\n{}",
            vertices_disp, faces_disp, halfedges_disp
        )
    }
}

/// A vertex of a DCEL
pub struct Vertex {
    /// (x, y) coordinates
    pub coordinates: Point,
    /// Some halfedge having this vertex as the origin
    pub incident_edge: usize, // index of halfedge
    /// False if the vertex has been deleted
    pub alive: bool,
}

impl fmt::Debug for Vertex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}, edge: {}", self.coordinates, self.incident_edge)
    }
}

/// A halfedge of a DCEL
pub struct HalfEdge {
    /// The index of the vertex at the start of the halfedge
    pub origin: usize, // index of vertex
    /// The index of the twin halfedge
    pub twin: usize, // index of halfedge
    /// The index of the next halfedge
    pub next: usize, // index of halfedge
    face: usize, // index of face
    prev: usize, // index of halfedge
    alive: bool,
}

impl fmt::Debug for HalfEdge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "origin: {}, twin: {}, next: {}",
            self.origin, self.twin, self.next
        )
    }
}

impl HalfEdge {
    /// Construct an empty halfedge
    pub fn new() -> Self {
        HalfEdge {
            origin: NIL,
            twin: NIL,
            next: NIL,
            face: NIL,
            prev: NIL,
            alive: true,
        }
    }
}

#[derive(Debug)]
/// A face of a DCEL
pub struct Face {
    outer_component: usize, // index of halfedge
    alive: bool,
}

impl fmt::Display for Face {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "outer: {}", self.outer_component)
    }
}

impl Face {
    /// Construct a new face, given an attached halfedge index
    pub fn new(edge: usize) -> Self {
        Face {
            outer_component: edge,
            alive: true,
        }
    }
}

fn add_twins_from_pt(start_pt: Point, dcel: &mut DCEL) -> (usize, usize, usize) {
    let (twin1, twin2) = dcel.add_twins();

    let start_vertex = Vertex {
        coordinates: start_pt,
        incident_edge: twin1,
        alive: true,
    };
    let start_vertex_ind = dcel.vertices.len();
    dcel.vertices.push(start_vertex);

    dcel.halfedges[twin1].origin = start_vertex_ind;

    (twin1, twin2, start_vertex_ind)
}

/// Construct faces for a DCEL.
///
/// # Panics
///
/// This method will panic if the DCEL has any faces already.
pub fn add_faces(dcel: &mut DCEL) {
    if !dcel.faces.is_empty() {
        panic!("add_faces only works on DCELs with no faces");
    }
    let num_halfedges = dcel.halfedges.len();
    let mut seen_edges = vec![false; num_halfedges];

    let mut processed_edges = 0;
    info!("Adding faces. There are {} halfedges.", num_halfedges);

    for edge_index in 0..num_halfedges {
        if seen_edges[edge_index] || !dcel.halfedges[edge_index].alive {
            continue;
        }
        processed_edges += 1;

        let face_index = dcel.faces.len();
        let new_face = Face::new(edge_index);
        dcel.faces.push(new_face);

        let mut current_edge = edge_index;
        loop {
            seen_edges[current_edge] = true;
            dcel.halfedges[current_edge].face = face_index;
            current_edge = dcel.halfedges[current_edge].next;
            if current_edge == edge_index {
                break;
            }
        }
    }
    info!("Generated faces for {} edges.", processed_edges);
}

/// Constructs the faces of monotone polygon subdivisions.
// WARN: make_polygons is not checked
pub fn make_polygons(dcel: &DCEL) -> Vec<Vec<Point>> {
    let mut result = vec![];
    for face in &dcel.faces {
        if !face.alive {
            continue;
        }
        let mut this_poly = vec![];
        let start_edge = face.outer_component;
        let mut current_edge = start_edge;
        loop {
            this_poly.push(dcel.get_origin(current_edge));
            current_edge = dcel.halfedges[current_edge].next;
            if current_edge == start_edge {
                break;
            }
        }
        result.push(this_poly);
    }

    // remove the outer face
    result.sort_by_key(|a| a.len());
    // ???: why remove the outer face
    // result.pop();

    result
}

/// Construsct a Dcel from a simple polygon in vec<Pos2>
pub fn polygon_to_dcel(polygon: &Vec<Point>) -> DCEL {
    let mut dcel = DCEL::new();
    // construct vertices
    let mut incident_edge_idx = 0;
    for point in polygon {
        let vertex = Vertex {
            coordinates: *point,
            incident_edge: incident_edge_idx,
            alive: true,
        };
        incident_edge_idx += 2;
        dcel.vertices.push(vertex);
    }
    // construct halfedges
    let i_max = dcel.vertices.len() - 1;
    for i in 0..dcel.vertices.len() {
        let mut he1 = HalfEdge::new(); // halfedge1
        let mut he2 = HalfEdge::new(); // the twin of halfedge1
        he1.origin = i;
        he2.origin = i + 1;
        he1.twin = 2 * i + 1;
        he2.twin = 2 * i;
        if i == 0 {
            // handle the first halfedge pair
            he1.next = 2 * i + 2;
            he2.next = 2 * i_max + 1;
            he1.prev = 2 * i_max;
            he2.prev = 2 * i + 3;
        } else if i == i_max {
            // handle the last halfedge pair
            he2.origin = 0;
            he1.next = 0;
            he2.next = 2 * i - 1;
            he1.prev = 2 * i - 2;
            he2.prev = 1;
        } else {
            he1.next = 2 * i + 2;
            he2.next = 2 * i - 1;
            he1.prev = 2 * i - 2;
            he2.prev = 2 * i + 3;
        }
        dcel.halfedges.push(he1);
        dcel.halfedges.push(he2);
    }
    // construct faces
    // ???: how to tell the halfedge added to the face
    // is a ccw or cw?
    let face = Face::new(0);
    dcel.faces.push(face);
    dcel
}

#[cfg(test)]
mod tests {
    use std::vec;

    use egui::Pos2 as Point;

    use super::{make_polygons, polygon_to_dcel};
    #[test]
    fn test_polygon_to_dcel() {
        let pts = vec![
            Point::new(1., 1.),
            Point::new(2., 2.),
            Point::new(1., 3.),
            Point::new(0., 2.),
        ];
        let dcel = polygon_to_dcel(&pts);
        // validate vertices
        let vertices = dcel
            .vertices
            .iter()
            .map(|v| v.coordinates)
            .collect::<Vec<Point>>();
        let incident_edge = dcel
            .vertices
            .iter()
            .map(|v| v.incident_edge)
            .collect::<Vec<usize>>();
        assert_eq!(vertices, pts);
        assert_eq!(incident_edge, [0, 2, 4, 6]);
        // validate halfedges
        let halfedges_origin = dcel
            .halfedges
            .iter()
            .map(|h| h.origin)
            .collect::<Vec<usize>>();
        assert_eq!(halfedges_origin, vec![0, 1, 1, 2, 2, 3, 3, 0]);
        let halfedges_twin = dcel
            .halfedges
            .iter()
            .map(|h| h.twin)
            .collect::<Vec<usize>>();
        assert_eq!(halfedges_twin, vec![1, 0, 3, 2, 5, 4, 7, 6]);
        let halfedges_next = dcel
            .halfedges
            .iter()
            .map(|h| h.next)
            .collect::<Vec<usize>>();
        assert_eq!(halfedges_next, vec![2, 7, 4, 1, 6, 3, 0, 5]);
        let halfedges_prev = dcel
            .halfedges
            .iter()
            .map(|h| h.prev)
            .collect::<Vec<usize>>();
        assert_eq!(halfedges_prev, vec![6, 3, 0, 5, 2, 7, 4, 1]);
        // validate faces
        let faces = dcel
            .faces
            .iter()
            .map(|f| f.outer_component)
            .collect::<Vec<usize>>();
        assert_eq!(faces, vec![0]);
    }

    #[test]
    fn test_make_polygon() {
        let pts = vec![
            Point::new(1., 1.),
            Point::new(2., 2.),
            Point::new(1., 3.),
            Point::new(0., 2.),
        ];
        let dcel = polygon_to_dcel(&pts);
        let polygon = make_polygons(&dcel);
        assert_eq!(polygon.len(), 1);
        assert_eq!(polygon[0], pts);
    }

    #[test]
    fn test_output_event_queue() {
        let truth = vec![0, 1, 3, 2];
        let pts = vec![
            Point::new(1., 1.),
            Point::new(2., 2.),
            Point::new(1., 3.),
            Point::new(0., 2.),
        ];
        let dcel = polygon_to_dcel(&pts);
        let order_queue = dcel.event_queue();
        assert_eq!(order_queue, truth);
    }
}
