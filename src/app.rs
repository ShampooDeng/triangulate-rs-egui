use eframe::egui::*;
use egui_extras::install_image_loaders;
use kd_tree::{KdMap, KdTree2};
use log::debug;
use std::iter::zip;

// use crate::monotone_y_partition::monoton_polygon_partition;
use crate::monotone_triangulation::polygon_triangulation;
use crate::monotone_y_partition::PartitionPolygon;
use crate::transform_pos::TransformPos;
use crate::vertex_coloring::dfs;

// #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
// #[cfg_attr(feature = "serde", serde(default))]

type Points = Vec<Pos2>;

#[allow(dead_code)]
fn example_poly() -> Points {
    vec![
        Pos2::new(157., 29.),  // 0
        Pos2::new(308., 173.), // 1
        Pos2::new(481., 49.),  // 2
        Pos2::new(624., 180.), // 3
        Pos2::new(500., 349.), // 4
        Pos2::new(378., 286.), // 5
        Pos2::new(185., 333.), // 6
    ]
}

fn example_poly2() -> Points {
    vec![
        Pos2::new(218., 60.),  //0
        Pos2::new(251., 197.), //1
        Pos2::new(362., 97.),  //2
        Pos2::new(460., 127.), //3
        Pos2::new(527., 250.), //4
        Pos2::new(628., 111.), //5
        Pos2::new(688., 38.),  //6
        Pos2::new(739., 257.), //7
        Pos2::new(646., 395.), //8
        Pos2::new(530., 406.), //9
        Pos2::new(380., 365.), //10
        Pos2::new(257., 413.), //11
    ]
}

fn generate_point_colors(len: usize) -> Vec<Color32> {
    vec![Color32::BLACK; len]
}
pub struct Painting {
    /// in 0-1 normalized coordinates
    points: Points,
    point_colors: Vec<egui::Color32>,
    polygon_partition: Vec<Points>,
    stroke: Stroke,
    radius: f32,
    kdtree: KdMap<[f32; 2], usize>,
    focused_point: Option<(Pos2, usize)>,
    _painting_rect: Rect,
    dcel: PartitionPolygon,

    // Application mode flag
    triangulated: bool,
    coloring: bool,

    // "about" page  window flag
    show_immediate_about_page: bool,
}

impl Default for Painting {
    fn default() -> Self {
        let points = example_poly2();
        Self {
            // points: Default::default(),
            points,
            point_colors: Vec::new(),
            polygon_partition: Vec::new(),
            stroke: Stroke::new(2.0, Color32::from_rgb(25, 200, 100)),
            radius: 5.,
            kdtree: KdTree2::default(),
            focused_point: None,
            _painting_rect: Rect {
                min: Pos2::ZERO,
                max: Pos2::ZERO,
            },
            dcel: PartitionPolygon::new(),

            triangulated: false,
            coloring: false,

            show_immediate_about_page: false,
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
    fn transpose_from_screen(&self) -> TransformPos {
        TransformPos::new(vec2(0., self._painting_rect.size().y), vec2(1., -1.))
    }

    /// Transpose coordinates from conventional coordinate system to gui's coordinate system.
    fn transpose_to_screen(&self) -> TransformPos {
        let from_screen = TransformPos::new(vec2(0., self._painting_rect.size().y), vec2(1., -1.));
        from_screen.inverse()
    }

    fn build_kd_tree(&mut self) {
        let face_iter = Vec::from_iter(self.dcel.faces.iter().map(|face| {
            let centrod = face.as_ref().borrow().centroid;
            [centrod.x, centrod.y]
        }));
        let face_index_iter = Vec::from_iter(0..self.dcel.faces.len());

        self.kdtree =
            KdMap::build_by_ordered_float(Vec::from_iter(zip(face_iter, face_index_iter)));
    }

    /// Draw vertices spawned by Mouse click in the drawing area.
    fn draw_vertices(&mut self, p: &Painter) {
        // Draw vertices
        let mut idx: usize = 0;
        let vertices = self.points.iter().map(|point| {
            // Transpose vertex coordinate to gui's coordiante system.
            let center = self.transpose_to_screen() * *point;
            let ret = if self.point_colors.is_empty() {
                egui::Shape::circle_filled(center, self.radius, Color32::BLACK)
            } else {
                egui::Shape::circle_filled(center, self.radius, self.point_colors[idx])
            };
            idx += 1;
            ret
        });
        p.extend(vertices);

        // Add number to lower right corner of the vertex
        for i in 0..self.points.len() {
            let font_id = egui::FontId::new(15., FontFamily::Monospace);
            let pt = self.transpose_to_screen() * self.points[i];
            let pos = pos2(pt.x + self.radius, pt.y + self.radius);
            let text = i.to_string();
            p.text(pos, Align2::LEFT_TOP, text, font_id, Color32::RED);
        }
    }

    fn draw_polygon(&self, pts: &Points, p: &Painter) {
        let mut points = pts
            .iter()
            // Transpose vertex coordinate to gui's coordiante system.
            .map(|point| self.transpose_to_screen() * *point)
            .collect::<Vec<Pos2>>();
        // Join the last vertex and the first vertex to seal the polygon.
        if self.points.len() > 2 {
            points.push(self.transpose_to_screen() * pts[0]);
        }
        let polygon_outline = Shape::line(points, self.stroke);
        p.add(polygon_outline);
    }

    fn draw_polygon_partition(&self, p: &Painter) {
        if self.polygon_partition.is_empty() {
            return;
        }
        for partition in self.polygon_partition.iter() {
            self.draw_polygon(partition, p);
        }
    }

    fn draw_centroid(&mut self, p: &Painter) {
        if !self.dcel.faces.is_empty() {
            for face in self.dcel.faces.iter() {
                let face_clone = face.clone();
                let centroid = self.transpose_to_screen() * face_clone.as_ref().borrow().centroid;

                let bounding_box_stroke = Stroke::new(2., Color32::BLACK);
                let rectangle = Rect {
                    max: pos2(centroid.x + self.radius, centroid.y + self.radius),
                    min: pos2(centroid.x - self.radius, centroid.y - self.radius),
                };
                let bounding_box =
                    egui::Shape::rect_stroke(rectangle, Rounding::ZERO, bounding_box_stroke);
                p.add(bounding_box);
            }
        }
    }

    fn draw_focused_point(&mut self, p: &Painter) {
        if let Some(focused_point) = self.focused_point {
            let centroid = self.transpose_to_screen() * focused_point.0;
            let rectangle = Rect {
                max: pos2(centroid.x + self.radius, centroid.y + self.radius),
                min: pos2(centroid.x - self.radius, centroid.y - self.radius),
            };
            let centroid_mark =
                egui::Shape::rect_filled(rectangle, egui::Rounding::ZERO, Color32::BLACK);
            p.add(centroid_mark);
        }
    }

    /// Define Gui widget layout, and button click event.
    fn ui_control(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.horizontal(|ui| {
            ui.label("Stroke:");
            ui.add(&mut self.stroke);
            ui.separator();
            ui.label("Radius");
            ui.add(DragValue::new(&mut self.radius));
            ui.separator();
            // Clear button
            if ui.button("Clear Painting").clicked() {
                self.points.clear();
                self.polygon_partition.clear();
                self.focused_point = None;
                self.dcel = PartitionPolygon::new();
                self.point_colors.clear();

                self.triangulated = false;
                self.coloring = false;
            }
            // triangle button
            let triangle_button = ui
                .add_enabled(!self.triangulated, egui::Button::new("Triangulate Polygon"))
                .on_hover_ui(|ui| {
                    ui.add(egui::widgets::Label::new(
                        "Check if the vertices in CCW order before click",
                    ));
                });
            if triangle_button.clicked() {
                self.triangulated = true;
                // self.polygon_partition = monoton_polygon_partition(&self.points);
                self.polygon_partition = polygon_triangulation(&self.points, &mut self.dcel);
            }
            // 3-coloring button
            if ui
                .add_enabled(self.triangulated, egui::Button::new("3-coloring triangles"))
                .clicked()
            {
                self.coloring = true;

                // Do 3 coloring vertices
                self.point_colors = generate_point_colors(self.points.len());
                let mut check_table: Vec<(usize, usize)> = Vec::new();
                let start_face_idx = if self.focused_point.is_none() {
                    0
                } else {
                    self.focused_point.unwrap().1
                };

                dfs(
                    self.dcel.faces[start_face_idx].clone(),
                    &mut check_table,
                    &mut self.point_colors,
                );
            }

            // Do something here
            if self.triangulated {
                ui.spinner();
            }
        })
        .response
    }

    /// Update gui elements
    fn ui_content(&mut self, ui: &mut Ui) -> egui::Response {
        let (mut response, painter) =
            ui.allocate_painter(ui.available_size_before_wrap(), Sense::click());
        self._painting_rect = response.rect;

        // Catch mouse click event
        if let Some(cur_pos) = response.interact_pointer_pos() {
            debug!("current cursor position:({},{})", cur_pos.x, cur_pos.y);
            // Transpose current cursor's position to conventional coordinate system.
            let current_point = self.transpose_from_screen() * cur_pos;

            // Define mouse click behavior in painting area.
            if self.triangulated {
                self.build_kd_tree();
                if let Some(nearest_point) =
                    self.kdtree.nearest(&[current_point.x, current_point.y])
                {
                    let [x, y] = nearest_point.item.0;
                    let face_idx = nearest_point.item.1;
                    self.focused_point = Some((pos2(x, y), face_idx));
                    debug!("Focused point coordinate:({},{})", x, y);
                }
            } else if let Some(last_point) = self.points.last() {
                // Reject the current cursor position that is too close the last point position.
                // HACK: Reject neighbor point that has same y coordinates,
                // This is a temperary solution to https://github.com/ShampooDeng/triangulate-rs-egui/issues/13
                if ((last_point.x - current_point.x).powi(2)
                    + (last_point.y - current_point.y).powi(2)
                    > 1000.)
                    && last_point.y.ne(&current_point.y)
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
        self.draw_polygon(&self.points, &painter);
        self.draw_polygon_partition(&painter);
        self.draw_vertices(&painter);
        self.draw_centroid(&painter);
        self.draw_focused_point(&painter);

        response
    }

    /// Draw ui content of the About page.
    fn render_about_page(&mut self, ctx: &egui::Context) {
        if !self.show_immediate_about_page {
            return;
        }

        // TODO: Is a there a easy way to only set font size for "about" page.
        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of("About"),
            egui::ViewportBuilder::default()
                .with_resizable(false)
                .with_inner_size([500.,300.])
                .with_title("About"),
            |ctx, class| {
                assert!(
                    class == egui::ViewportClass::Immediate,
                    "Current backend dones't support multiple viewport"
                );
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.strong("Triangulate-rs-egui");
                        ui.label("0.2.0");
                        ui.label("A polygon triangulation demonstration app based on egui, a rust gui framework.");

                        install_image_loaders(ctx);
                        // The path in include_image will be resolved at 
                        // compile-time and embedded in the binary. 
                        // ui.image(egui::include_image!("../assets/dadparrot.gif"));
                        ui.add(egui::Image::new(
                                egui::include_image!("../assets/tripletsparrot.gif")
                            ).max_width(100.)
                            .rounding(20.)
                        );

                        ui.label("Created by:");
                        ui.strong("ShampooDeng <lwxkkdy@foxmail.com>");

                        ui.label("Report a issue:");
                        use egui::special_emojis::GITHUB;
                        ui.hyperlink_to(format!("{GITHUB}triangulate-rs-egui"), 
                            "https://github.com/ShampooDeng/triangulate-rs-egui/issues");
                    })
                });

                if ctx.input(|i| i.viewport().close_requested()) {
                    // Tell parent viewport that we should not show next frame:
                    self.show_immediate_about_page = false;
                }
            }
        );
    }
}

impl eframe::App for Painting {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                egui::widgets::global_dark_light_mode_buttons(ui);
                if ui.button("About").clicked() {
                    self.show_immediate_about_page = true;
                }
            });
            self.render_about_page(ctx);
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            if self.triangulated {
                ui.heading("Click to select a rectangle!");
            } else {
                ui.horizontal(|ui| {
                    ui.heading("Click to place vertex! MUST in ");
                    ui.heading(RichText::new("Counter Clock-Wise").strong());
                    ui.heading("order");
                });
            }
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
        let points = [pos2(1., 1.), pos2(2., 2.), pos2(3., 1.)];
        let kdtree = kd_tree::KdTree2::build_by_ordered_float(Vec::from_iter(
            points.iter().map(|point| [point.x, point.y]),
        ));
        assert_eq!(kdtree.nearest(&[1., 1.1]).unwrap().item, &[1., 1.]);
    }
}
