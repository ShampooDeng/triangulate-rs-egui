use crate::monotone_y_partition::Face;
use egui::Color32;
use log::debug;
use std::{cell::RefCell, rc::Rc};

fn coloring_triangle(face: Rc<RefCell<Face>>, colors: &mut Vec<Color32>) {
    let mut red_avaiable = true;
    let mut green_avaiable = true;
    let mut blue_avaiable = true;
    let mut process_stack: Vec<usize> = Vec::new();
    for i in 0..3 {
        let idx = face.as_ref().borrow().vertices[i];
        let vertex_color = colors[idx];
        if let Color32::BLACK = vertex_color {
            process_stack.push(idx);
        } else if let Color32::RED = vertex_color {
            red_avaiable = false;
        } else if let Color32::GREEN = vertex_color {
            green_avaiable = false;
        } else if let Color32::BLUE = vertex_color {
            blue_avaiable = false;
        }
    }

    for idx in process_stack.iter() {
        if red_avaiable {
            colors[*idx] = Color32::RED;
            red_avaiable = false;
        } else if green_avaiable {
            colors[*idx] = Color32::GREEN;
            green_avaiable = false;
        } else if blue_avaiable {
            colors[*idx] = Color32::BLUE;
            blue_avaiable = false;
        }
    }
}

pub fn dfs(
    start_face: Rc<RefCell<Face>>,
    half_diag_check_table: &mut Vec<(usize, usize)>,
    colors: &mut Vec<Color32>,
) {
    let face_clone1 = start_face.clone();
    let mut unchecked_diags = start_face.as_ref().borrow().bounding_diags.len();

    let face_clone2 = start_face.clone();
    debug!("on parition{:?}", face_clone2.as_ref().borrow().vertices);
    coloring_triangle(start_face.clone(), colors);

    // Traverse triangulation partition recursively
    loop {
        if unchecked_diags == 0 {
            break;
        }

        let half_diag =
            face_clone1.as_ref().borrow_mut().bounding_diags[unchecked_diags - 1].clone();
        let twin = half_diag.as_ref().borrow().twin.clone().unwrap().clone();
        half_diag_check_table.push((
            half_diag.as_ref().borrow().origin,
            half_diag.as_ref().borrow().end,
        ));

        if !half_diag_check_table
            .contains(&(twin.as_ref().borrow().origin, twin.as_ref().borrow().end))
        {
            dfs(
                twin.as_ref()
                    .borrow()
                    .bounding_face
                    .clone()
                    .unwrap()
                    .clone(),
                half_diag_check_table,
                colors,
            );
        }
        unchecked_diags -= 1;
    }

    return;
}
