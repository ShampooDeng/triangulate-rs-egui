use eframe::egui::*;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct Painting {
    /// in 0-1 normalized coordinates
    points: Vec<Pos2>,
    stroke: Stroke,
    radius: f32,
    triangulating: bool,
}

impl Default for Painting {
    fn default() -> Self {
        Self {
            points: Default::default(),
            stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
            radius: 5.,
            triangulating: false,
        }
    }
}

impl Painting {
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

    fn _timer(&mut self) {
        todo!()
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
            if self.triangulating {
                ui.spinner();
            }
        })
        .response
    }

    fn ui_content(&mut self, ui: &mut Ui) -> egui::Response {
        let (mut response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::click());

        if let Some(pointer_pos) = response.interact_pointer_pos() {
            if let Some(last_point) = self.points.last() {
                if (last_point.x - pointer_pos.x).powi(2) + (last_point.y - pointer_pos.y).powi(2)
                    > 1000.
                {
                    self.points.push(pointer_pos);
                    response.mark_changed();
                }
            } else {
                self.points.push(pointer_pos);
                response.mark_changed();
            }
        }

        self.draw_vertices(&painter);
        self.draw_polygon(&painter);

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