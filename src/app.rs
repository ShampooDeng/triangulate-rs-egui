use eframe::egui::*;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct Painting {
    /// in 0-1 normalized coordinates
    points: Vec<Pos2>,
    stroke: Stroke,
    radius: f32,
    kdtree: kd_tree::KdTree2<[f32; 2]>,
    foucsed_point: Pos2,

    triangulating: bool,
    coloring: bool,
}

impl Default for Painting {
    fn default() -> Self {
        Self {
            points: Default::default(),
            stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
            radius: 5.,
            kdtree: kd_tree::KdTree2::default(),
            foucsed_point: pos2(-1., -1.),

            // App mode flag
            triangulating: false,
            coloring: false,
        }
    }
}

impl Painting {
    fn mark_selected_point(&mut self, p: &Painter) {
        if self.coloring {
            let bounding_box_stroke = Stroke::new(2., Color32::BLACK);
            let rectangle = Rect {
                max: pos2(
                    self.foucsed_point.x + self.radius,
                    self.foucsed_point.y + self.radius,
                ),
                min: pos2(
                    self.foucsed_point.x - self.radius,
                    self.foucsed_point.y - self.radius,
                ),
            };
            let bounding_box = egui::Shape::rect_stroke(rectangle, Rounding::ZERO, bounding_box_stroke);
            p.add(bounding_box);
        }
    }

    fn draw_vertices(&mut self, p: &Painter) {
        let vertices = self.points.iter().map(|point| {
            let center = *point;
            egui::Shape::circle_filled(center, self.radius, Color32::RED)
        });
        p.extend(vertices);
    }

    fn draw_polygon(&mut self, p: &Painter) {
        let mut points = self.points.clone();
        if self.points.len() > 2 {
            points.push(self.points[0]);
        }
        let polygon_outline = Shape::line(points, self.stroke);
        p.add(polygon_outline);
    }

    fn ui_control(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            ui.label("Stroke:");
            egui::stroke_ui(ui, &mut self.stroke, "preview");
            ui.separator();
            ui.label("Radius");
            ui.add(DragValue::new(&mut self.radius));
            ui.separator();
            if ui.button("Clear Painting").clicked() {
                self.points.clear();
            }
            if ui.button("Triangulate Polygon").clicked() {
                self.triangulating = true;
                // TODO: add text to info user that "Triangulation is in process"
            }
            if ui.button("3-coloring triangles").clicked() {
                self.coloring = !self.coloring;
            }
            if self.triangulating {
                ui.spinner();
            }
        })
        .response
    }

    fn ui_content(&mut self, ui: &mut Ui) -> egui::Response {
        let (mut response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::click());

        if let Some(current_pos) = response.interact_pointer_pos() {
            if self.coloring {
                self.kdtree = kd_tree::KdTree2::build_by_ordered_float(Vec::from_iter(
                    self.points.iter().map(|point| [point.x, point.y]),
                ));
                if let Some(nearest_point) = self.kdtree.nearest(&[current_pos.x, current_pos.y]) {
                    let x = nearest_point.item[0];
                    let y = nearest_point.item[1];
                    self.foucsed_point = pos2(x, y);
                }
            } else if self.triangulating {
                // TODO: implement triangulate algorithm
                // TOOD: demonstrate the process of triangulating step by step.
                todo!()
            } else if let Some(last_point) = self.points.last() {
                // Reject the current cursor position is too close the last point position.
                if (last_point.x - current_pos.x).powi(2) + (last_point.y - current_pos.y).powi(2)
                    > 1000.
                {
                    self.points.push(current_pos);
                    response.mark_changed();
                }
            } else {
                // Jump to here when the points vec is empty.
                self.points.push(current_pos);
                response.mark_changed();
            }
        }

        self.draw_vertices(&painter);
        self.draw_polygon(&painter);
        self.mark_selected_point(&painter);

        response
    }
}

impl eframe::App for Painting {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::widgets::global_dark_light_mode_buttons(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Painting!");
            self.ui_control(ui);
            self.ui_content(ui);
        });
    }
}
