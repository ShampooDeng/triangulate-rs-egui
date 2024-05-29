use eframe::egui::*;
use kd_tree::KdTree2;
use log::debug;

use crate::dcel::polygon_to_dcel;
use crate::triangulate::make_monotone;
use crate::TransformPos;

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
    _painting_rect: Rect,

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
            focused_point: pos2(-1., -1.), // ???: Is there a better choice than (-1., -1.)
            _painting_rect: Rect {
                min: Pos2::ZERO,
                max: Pos2::ZERO,
            },

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

    /// Transpose coordinates from gui's coordinate system to conventional coordinate system.
    /// Gui's coordinate system has its origin in the top left corner, while
    /// conventional coordiante system's origin rests in the lower left corner,
    /// which is more intuitive and easier to handle.
    fn from_screen(&self) -> TransformPos {
        let from_screen = TransformPos::new(vec2(0., self._painting_rect.size().y), vec2(1., -1.));
        from_screen
    }

    /// Transpose coordinates from conventional coordinate system to gui's coordinate system.
    fn to_screen(&self) -> TransformPos {
        let from_screen = TransformPos::new(vec2(0., self._painting_rect.size().y), vec2(1., -1.));
        from_screen.inverse()
    }

    fn mark_selected_point(&mut self, p: &Painter) {
        if self.coloring {
            let bounding_box_stroke = Stroke::new(2., Color32::BLACK);
            let focused_pt = self.to_screen() * self.focused_point;
            let rectangle = Rect {
                max: pos2(focused_pt.x + self.radius, focused_pt.y + self.radius),
                min: pos2(focused_pt.x - self.radius, focused_pt.y - self.radius),
            };
            let bounding_box =
                egui::Shape::rect_stroke(rectangle, Rounding::ZERO, bounding_box_stroke);
            p.add(bounding_box);
        }
    }

    fn draw_vertices(&mut self, p: &Painter) {
        // Draw vertices
        let vertices = self.points.iter().map(|point| {
            // Transpose vertex coordinate to gui's coordiante system.
            let center = self.to_screen() * *point;
            egui::Shape::circle_filled(center, self.radius, Color32::RED)
        });
        p.extend(vertices);
        // Add number to lower right corner of the vertex
        for i in 0..self.points.len() {
            let font_id = egui::FontId::new(15., FontFamily::Monospace);
            let pt = self.to_screen() * self.points[i];
            let pos = pos2(pt.x + self.radius, pt.y + self.radius);
            let text = i.to_string();
            p.text(pos, Align2::LEFT_TOP, text, font_id, Color32::BLACK);
        }
    }

    fn draw_polygon(&mut self, p: &Painter) {
        // TODO: use the polygons generated from dcel to draw polygons
        let mut points = self
            .points
            .iter()
            .map(|point| self.to_screen() * *point)// Transpose vertex coordinate to gui's coordiante system.
            .collect::<Vec<Pos2>>();
        // Join the last vertex and the first vertex to seal the polygon.
        if self.points.len() > 2 {
            points.push(self.to_screen() * self.points[0]);
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
                self.focused_point = pos2(-1., -1.);
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
        self._painting_rect = response.rect;
        if let Some(cur_pos) = response.interact_pointer_pos() {
            debug!(
                "current cursor position:({},{})",
                cur_pos.x, cur_pos.y
            );
            // Transpose current cursor's position to conventional coordinate system.
            let current_point = self.from_screen() * cur_pos;
            if self.coloring {
                self.kdtree = KdTree2::build_by_ordered_float(Vec::from_iter(
                    self.points.iter().map(|point| [point.x, point.y]),
                ));
                if let Some(nearest_point) = self.kdtree.nearest(&[current_point.x, current_point.y]) {
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
                if (last_point.x - current_point.x).powi(2)
                    + (last_point.y - current_point.y).powi(2)
                    > 1000.
                {
                    self.points.push(current_point);
                    response.mark_changed();
                    debug!(
                        "Pushing point coordinate:({},{})",
                        self.points.last().unwrap().x,
                        self.points.last().unwrap().y
                    );
                }
            } else {
                // Jump to here when the points vec is empty.
                self.points.push(current_point);
                debug!(
                    "Pushing point coordinate:({},{})",
                    self.points.last().unwrap().x,
                    self.points.last().unwrap().y
                );
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
