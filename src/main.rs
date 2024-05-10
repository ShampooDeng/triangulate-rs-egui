#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example
#![allow(clippy::undocumented_unsafe_blocks)]

use eframe::egui::*;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct Painting {
    /// in 0-1 normalized coordinates
    points: Vec<Pos2>,
    stroke: Stroke,
}

impl Default for Painting {
    fn default() -> Self {
        Self {
            points: Default::default(),
            stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
        }
    }
}

impl Painting {
    pub fn ui_control(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            ui.label("Stroke:");
            egui::stroke_ui(ui, &mut self.stroke, "nihao");
            ui.separator();
            if ui.button("Clear Painting").clicked() {
                self.points.clear();
            }
        })
        .response
    }

    pub fn ui_content(&mut self, ui: &mut Ui) -> egui::Response {
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

        let shapes = self
            .points
            .iter()
            // .filter(|point| point.len() >= 1)
            .map(|point| {
                // let center = to_screen * *point;
                let center = *point;
                egui::Shape::circle_filled(center, 5., Color32::RED)
            });

        painter.extend(shapes);

        response
    }
}

impl eframe::App for Painting {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::widgets::global_dark_light_mode_buttons(ui);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("painting");
            self.ui_control(ui);
            self.ui_content(ui);
        });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };
    eframe::run_native(
        "egui painting plate",
        options,
        Box::new(|_cc| Box::<Painting>::default()),
    )?;
    Ok(())
}
