use eframe::egui::*;
use kd_tree::KdTree2;
use log::debug;

use crate::dcel::polygon_to_dcel;
use crate::triangulate::make_monotone;

// TODO: more detailed comments

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct Painting {
    /// in 0-1 normalized coordinates
    points: Vec<Pos2>,
    stroke: Stroke,
    radius: f32,
    kdtree: KdTree2<[f32; 2]>,
    focused_point: Pos2,

    // Application mode flag
    triangulating: bool,
    coloring: bool,
}

impl Default for Painting {
    fn default() -> Self {
        Self {
            points: Default::default(),
            stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
            radius: 5.,
            kdtree: KdTree2::default(),
            focused_point: pos2(-1., -1.),

            triangulating: false,
            coloring: false,
        }
    }
}

impl Painting {
    /// Called once before the first frame.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        // }

        Default::default()
    }

    fn mark_selected_point(&mut self, p: &Painter) {
        if self.coloring {
            let bounding_box_stroke = Stroke::new(2., Color32::BLACK);
            let rectangle = Rect {
                max: pos2(
                    self.focused_point.x + self.radius,
                    self.focused_point.y + self.radius,
                ),
                min: pos2(
                    self.focused_point.x - self.radius,
                    self.focused_point.y - self.radius,
                ),
            };
            let bounding_box =
                egui::Shape::rect_stroke(rectangle, Rounding::ZERO, bounding_box_stroke);
            p.add(bounding_box);
        }
    }

    fn draw_vertices(&mut self, p: &Painter) {
        // Draw vertices
        let vertices = self.points.iter().map(|point| {
            let center = *point;
            egui::Shape::circle_filled(center, self.radius, Color32::RED)
        });
        p.extend(vertices);
        // Add number to lower right corner of the vertex
        for i in 0..self.points.len() {
            let font_id = egui::FontId::new(15., FontFamily::Monospace);
            let pos = pos2(self.points[i].x + self.radius, self.points[i].y + self.radius);
            let text = i.to_string();
            p.text(pos, Align2::LEFT_TOP, text, font_id, Color32::BLACK);
        }
    }

    fn draw_polygon(&mut self, p: &Painter) {
        // TODO: use the polygons generated from dcel to draw polygons
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
                // TODO: add text to inform user that "Triangulation is in process"
                let dcel = polygon_to_dcel(&self.points);
                make_monotone(&dcel);
                self.triangulating = false;
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
            debug!(
                "current cursor position:({},{})",
                current_pos.x, current_pos.y
            );
            if self.coloring {
                self.kdtree = KdTree2::build_by_ordered_float(Vec::from_iter(
                    self.points.iter().map(|point| [point.x, point.y]),
                ));
                if let Some(nearest_point) = self.kdtree.nearest(&[current_pos.x, current_pos.y]) {
                    let x = nearest_point.item[0];
                    let y = nearest_point.item[1];
                    self.focused_point = pos2(x, y);
                    debug!(
                        "Focused point coordinate:({},{})",
                        self.focused_point.x, self.focused_point.y
                    );
                }
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

        // Drawing ui content
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

#[cfg(test)]
mod tests {
    use egui::pos2;

    #[test]
    fn test_kdtree() {
        let points = vec![pos2(1., 1.), pos2(2., 2.), pos2(3., 1.)];
        let kdtree = kd_tree::KdTree2::build_by_ordered_float(Vec::from_iter(
            points.iter().map(|point| [point.x, point.y]),
        ));
        assert_eq!(kdtree.nearest(&[1., 1.1]).unwrap().item, &[1., 1.]);
    }
}
