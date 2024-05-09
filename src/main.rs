// #![warn(clippy::all, rust_2018_idioms)]
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// // When compiling natively:
// // #[cfg(not(target_arch = "wasm32"))]
// fn main() -> eframe::Result<()> {
//     env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

//     let native_options = eframe::NativeOptions {
//         viewport: egui::ViewportBuilder::default()
//             .with_inner_size([400.0, 300.0])
//             .with_min_inner_size([300.0, 220.0])
//             .with_icon(
//                 // NOTE: Adding an icon is optional
//                 eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
//                     .expect("Failed to load icon"),
//             ),
//         ..Default::default()
//     };
//     eframe::run_native(
//         "eframe template",
//         native_options,
//         Box::new(|cc| Box::new(try_egui::TemplateApp::new(cc))),
//     )
// }
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // it's an example
#![allow(clippy::undocumented_unsafe_blocks)]

// Test that we can paint to the screen using glow directly.

// use eframe::egui;
// use eframe::glow;

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
//     let options = eframe::NativeOptions {
//         renderer: eframe::Renderer::Glow,
//         ..Default::default()
//     };
//     eframe::run_native(
//         "My test app",
//         options,
//         Box::new(|_cc| Box::<MyTestApp>::default()),
//     )?;
//     Ok(())
// }

// #[derive(Default)]
// struct MyTestApp {}

// impl eframe::App for MyTestApp {
//     fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
//         use glow::HasContext as _;
//         let gl = frame.gl().unwrap();

//         #[allow(unsafe_code)]
//         unsafe {
//             gl.disable(glow::SCISSOR_TEST);
//             gl.viewport(0, 0, 100, 100);
//             gl.clear_color(1.0, 0.0, 1.0, 1.0); // purple
//             gl.clear(glow::COLOR_BUFFER_BIT);
//         }

//         egui::Window::new("Floating Window").show(ctx, |ui| {
//             ui.label("The background should be purple.");
//         });
//     }
// }

use eframe::egui::*;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct Painting {
    /// in 0-1 normalized coordinates
    lines: Vec<Vec<Pos2>>,
    stroke: Stroke,
}

impl Default for Painting {
    fn default() -> Self {
        Self {
            lines: Default::default(),
            stroke: Stroke::new(1.0, Color32::from_rgb(25, 200, 100)),
        }
    }
}

impl Painting {
    pub fn ui_control(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            ui.label("Stroke:");
            // ui.add(&mut self.stroke);
            egui::stroke_ui(ui, &mut self.stroke, "nihao");
            ui.separator();
            if ui.button("Clear Painting").clicked() {
                self.lines.clear();
            }
        })
        .response
    }

    pub fn ui_content(&mut self, ui: &mut Ui) -> egui::Response {
        let (mut response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::drag());

        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2::ZERO, response.rect.square_proportions()),
            response.rect,
        );
        let from_screen = to_screen.inverse();

        if self.lines.is_empty() {
            self.lines.push(vec![]);
        }

        let current_line = self.lines.last_mut().unwrap();

        if let Some(pointer_pos) = response.interact_pointer_pos() {
            let canvas_pos = from_screen * pointer_pos;
            if current_line.last() != Some(&canvas_pos) {
                current_line.push(canvas_pos);
                response.mark_changed();
            }
        } else if !current_line.is_empty() {
            self.lines.push(vec![]);
            response.mark_changed();
        }

        let shapes = self
            .lines
            .iter()
            .filter(|line| line.len() >= 2)
            .map(|line| {
                let points: Vec<Pos2> = line.iter().map(|p| to_screen * *p).collect();
                egui::Shape::line(points, self.stroke)
            });

        painter.extend(shapes);

        response
    }
}

impl eframe::App for Painting {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui|{
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