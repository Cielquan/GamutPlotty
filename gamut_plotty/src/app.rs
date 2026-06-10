use color_calc::CIELAB;
use egui::{
    Align, AtomExt, Button, CentralPanel, Color32, ComboBox, DragValue, Frame, Image, IntoAtoms,
    Label, Layout, MenuBar, Panel, Pos2, Sense, Theme, ThemePreference, include_image,
};

use crate::dummy_state::create_color_points;
use crate::gamut_data;

const APP_NAME: &str = "Gamut Plotty";
const APP_KEY: &str = "gamut_plotty_app";

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct GamutPlottyApp {
    color_points: Vec<CIELAB::LabPoint>,
    selected_illuminant: gamut_data::Illuminant,
    selected_observer: gamut_data::Observer,
    camera_settings: CameraSettings,
}

impl Default for GamutPlottyApp {
    fn default() -> Self {
        Self {
            color_points: create_color_points(),
            selected_illuminant: gamut_data::Illuminant::default(),
            selected_observer: gamut_data::Observer::default(),
            camera_settings: CameraSettings::default(),
        }
    }
}

const MIN_ZOOM: f32 = 0.1;
const MAX_ZOOM: f32 = 1000.0;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct CameraSettings {
    distance: f32,
    height: f32,
    fov: f32,
    rotation_sensitivity: f32,
    #[serde(skip)]
    rotation: glam::Quat,
    #[serde(skip)]
    zoom: f32,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            distance: 250.0,
            height: 50.0,
            rotation_sensitivity: 0.01,
            rotation: glam::Quat::default(),
            zoom: 100.0,
            fov: 1.0, // Roughly 60 degrees
        }
    }
}

impl GamutPlottyApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }

    fn convert_3d_to_2d(
        &self,
        points: Vec<glam::Vec3>,
        center_position: Pos2,
    ) -> Vec<Option<Pos2>> {
        let camera_position = glam::Vec3::new(0.0, self.camera_settings.height, 0.0);

        points
            .iter()
            .map(|&point| {
                let translated_point = point - camera_position;
                let rotated_point = self.camera_settings.rotation * translated_point;
                // Translate (Move camera back)
                // We add a Z offset so the object is in front of the camera (0,0,0)
                let z = rotated_point.z + self.camera_settings.distance;

                // Clip (Don't draw if behind camera)
                if z <= 0.1 {
                    return None;
                }

                let scale = self.camera_settings.zoom * self.camera_settings.fov / z;
                let x = rotated_point.x * scale;
                let y = rotated_point.y * scale;
                // Map to screen coordinates (Flip Y because screen Y is down)
                Some(Pos2::new(center_position.x + x, center_position.y - y))
            })
            .collect()
    }

    fn convert_3d_to_2d_with_vis_map(
        &self,
        points: Vec<glam::Vec3>,
        center_position: Pos2,
    ) -> (Vec<Pos2>, Vec<bool>) {
        let camera_position = glam::Vec3::new(0.0, self.camera_settings.height, 0.0);

        let mut projected_coords = Vec::with_capacity(points.len());
        let mut visibility_map = Vec::with_capacity(points.len());

        for &point in &points {
            let translated_point = point - camera_position;
            let rotated_point = self.camera_settings.rotation * translated_point;
            // Translate (Move camera back)
            // We add a Z offset so the object is in front of the camera (0,0,0)
            let z = rotated_point.z + self.camera_settings.distance;

            if z > 0.1 {
                let scale = self.camera_settings.zoom * self.camera_settings.fov / z;
                let x = rotated_point.x * scale;
                let y = rotated_point.y * scale;

                // Map to screen coordinates (Flip Y because screen Y is down)
                projected_coords.push(Pos2::new(center_position.x + x, center_position.y - y));
                visibility_map.push(true);
            } else {
                // Add a dummy point to maintain alignment
                projected_coords.push(Pos2::new(center_position.x, center_position.y));
                visibility_map.push(false);
            }
        }

        (projected_coords, visibility_map)
    }

    fn draw_axis_segment(
        &self,
        painter: &egui::Painter,
        points: Vec<glam::Vec3>,
        center: egui::Pos2,
        color: egui::Color32,
    ) {
        let mut prev_visible_point: Option<egui::Pos2> = None;
        for point in self.convert_3d_to_2d(points, center) {
            if let Some(point_val) = point {
                // If we have a previous visible point, draw the line
                if let Some(prev) = prev_visible_point {
                    painter.line_segment([prev, point_val], egui::Stroke::new(1.0, color));
                }
                // Update previous
                prev_visible_point = Some(point_val);
            } else {
                // Point is behind camera. Break the line.
                // Next time we see a point, we start a NEW segment, not connecting to the old one.
                prev_visible_point = None;
            }
        }
    }

    // TODO: remove
    fn draw_cube_frame(&self, painter: &egui::Painter, center: egui::Pos2) {
        let min_x = -200.0;
        let max_x = 200.0;
        let min_y = 0.0;
        let max_y = 110.0;
        let min_z = -200.0;
        let max_z = 200.0;

        // Define the 8 corners of the cube
        let corners = vec![
            glam::Vec3::new(min_x, min_y, min_z), // 0: Bottom-Front-Left
            glam::Vec3::new(max_x, min_y, min_z), // 1: Bottom-Front-Right
            glam::Vec3::new(max_x, min_y, max_z), // 2: Bottom-Back-Right
            glam::Vec3::new(min_x, min_y, max_z), // 3: Bottom-Back-Left
            glam::Vec3::new(min_x, max_y, min_z), // 4: Top-Front-Left
            glam::Vec3::new(max_x, max_y, min_z), // 5: Top-Front-Right
            glam::Vec3::new(max_x, max_y, max_z), // 6: Top-Back-Right
            glam::Vec3::new(min_x, max_y, max_z), // 7: Top-Back-Left
        ];

        // Define the 12 edges by index pairs
        let edges = [
            // Bottom face
            (0, 1),
            (1, 2),
            (2, 3),
            (3, 0),
            // Top face
            (4, 5),
            (5, 6),
            (6, 7),
            (7, 4),
            // Vertical connections
            (0, 4),
            (1, 5),
            (2, 6),
            (3, 7),
        ];

        let projected = self.convert_3d_to_2d(corners, center);

        let color = Color32::from_rgb(200, 200, 200);
        let line_thickness = 1.5;

        // Draw edges
        for (start_idx, end_idx) in edges {
            if let (Some(p1), Some(p2)) = (projected[start_idx], projected[end_idx]) {
                painter.line_segment([p1, p2], egui::Stroke::new(line_thickness, color));
            }
            // If either point is behind camera, we simply don't draw that edge segment.
            // This creates a clean "broken" look rather than stretching across the screen.
        }
    }
}

impl eframe::App for GamutPlottyApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        Panel::top("top_panel").show_inside(ui, |ui| {
            // The top panel is often a good place for a menu bar:

            MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ui.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                let mut theme_preference = ui.options(|opt| opt.theme_preference);
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    ui.horizontal(|ui| {
                        let current_system_theme =
                            if let Some(system_theme) = ui.input(|i| i.raw.system_theme) {
                                match system_theme {
                                    Theme::Dark => "dark",
                                    Theme::Light => "light",
                                }
                            } else {
                                "unknown"
                            };

                        fn theme_button<'a, Value: PartialEq>(
                            ui: &mut egui::Ui,
                            current_value: &mut Value,
                            selected_value: Value,
                            contents: impl IntoAtoms<'a>,
                        ) -> egui::Response {
                            let btn = Button::new(contents)
                                .selected(*current_value == selected_value)
                                .frame_when_inactive(*current_value == selected_value)
                                .image_tint_follows_text_color(true)
                                .frame(true);
                            let mut response = ui.add(btn);
                            if response.clicked() && *current_value != selected_value {
                                *current_value = selected_value;
                                response.mark_changed();
                            }
                            response
                        }

                        theme_button(
                            ui,
                            &mut theme_preference,
                            ThemePreference::Light,
                            (
                                Image::new(include_image!("../../assets/images/sun.svg"))
                                    .atom_max_height_font_size(ui),
                                "Light",
                            ),
                        )
                        .on_hover_text("Use light mode");

                        theme_button(
                            ui,
                            &mut theme_preference,
                            ThemePreference::Dark,
                            (
                                Image::new(include_image!("../../assets/images/moon.svg")),
                                "Dark",
                            ),
                        )
                        .on_hover_text("Use dark mode");

                        theme_button(
                            ui,
                            &mut theme_preference,
                            ThemePreference::System,
                            (
                                Image::new(include_image!("../../assets/images/sun-moon.svg")),
                                "System",
                            ),
                        )
                        .on_hover_ui(|ui| {
                            ui.label("Follow system theme");
                            ui.add_space(4.0);
                            ui.label(format!(
                                "The current system theme is: {current_system_theme}"
                            ));
                        });
                    });
                    ui.ctx().set_theme(theme_preference);

                    egui::warn_if_debug_build(ui);
                });
            });
        });

        CentralPanel::default().show_inside(ui, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading(APP_NAME);

            ui.separator();

            ui.horizontal(|ui| {
                ComboBox::from_label("Illuminant")
                    .selected_text(self.selected_illuminant.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.selected_illuminant,
                            gamut_data::Illuminant::D50,
                            gamut_data::Illuminant::D50.to_string(),
                        );
                        ui.selectable_value(
                            &mut self.selected_illuminant,
                            gamut_data::Illuminant::D65,
                            gamut_data::Illuminant::D65.to_string(),
                        );
                    });

                ComboBox::from_label("Observer")
                    .selected_text(self.selected_observer.to_string())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.selected_observer,
                            gamut_data::Observer::CIE2deg1931,
                            gamut_data::Observer::CIE2deg1931.to_string(),
                        );
                        ui.selectable_value(
                            &mut self.selected_observer,
                            gamut_data::Observer::CIE2deg2015,
                            gamut_data::Observer::CIE2deg2015.to_string(),
                        );
                        ui.selectable_value(
                            &mut self.selected_observer,
                            gamut_data::Observer::CIE10deg1964,
                            gamut_data::Observer::CIE10deg1964.to_string(),
                        );
                        ui.selectable_value(
                            &mut self.selected_observer,
                            gamut_data::Observer::CIE10deg2015,
                            gamut_data::Observer::CIE10deg2015.to_string(),
                        );
                    });
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.add(Label::new("Distance:"));
                ui.add(DragValue::new(&mut self.camera_settings.distance).speed(1.0));

                ui.separator();

                ui.add(Label::new("Height:"));
                ui.add(DragValue::new(&mut self.camera_settings.height).speed(1.0));

                ui.separator();

                ui.add(Label::new("Rot. sen.:"));
                ui.add(DragValue::new(&mut self.camera_settings.rotation_sensitivity).speed(0.01));

                ui.separator();

                ui.add(Label::new("Zoom:"));
                ui.add(DragValue::new(&mut self.camera_settings.zoom).speed(1.0));

                ui.separator();

                ui.add(Label::new("FOV:"));
                ui.add(DragValue::new(&mut self.camera_settings.fov).speed(0.01));

                ui.separator();

                if ui.add(Button::new("Reset")).clicked() {
                    self.camera_settings = CameraSettings::default();
                }
            });

            ui.separator();

            Frame::group(ui.style()).show(ui, |ui| {
                let group_rect = ui.response().rect;
                let group_center = group_rect.center();
                let painter = ui.painter().with_clip_rect(group_rect);

                let response = ui.allocate_response(ui.available_size(), Sense::drag());
                let is_hovered = response.hovered();

                if is_hovered {
                    let scroll_delta = ui.ctx().input(|i| i.smooth_scroll_delta);
                    if scroll_delta.y != 0.0 {
                        // positiv > scroll up; negative > scroll down
                        let factor = 1.0 + scroll_delta.y * 0.001;
                        self.camera_settings.zoom =
                            (self.camera_settings.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
                    }
                }

                if is_hovered && response.dragged() {
                    let delta = response.drag_delta();
                    let rot_y = glam::Quat::from_rotation_y(
                        -delta.x * self.camera_settings.rotation_sensitivity,
                    );
                    let rot_x = glam::Quat::from_rotation_x(
                        -delta.y * self.camera_settings.rotation_sensitivity,
                    );
                    self.camera_settings.rotation = rot_y * rot_x * self.camera_settings.rotation;
                }

                // X axis (a+)
                {
                    let points: Vec<glam::Vec3> = (0..=200)
                        .map(|i| glam::Vec3::new(i as f32, self.camera_settings.height, 0.0))
                        .collect();
                    self.draw_axis_segment(&painter, points, group_center, Color32::RED);
                }

                // X axis (a-)
                {
                    let points: Vec<glam::Vec3> = (-200..=0)
                        .map(|i| glam::Vec3::new(i as f32, self.camera_settings.height, 0.0))
                        .collect();
                    self.draw_axis_segment(&painter, points, group_center, Color32::GREEN);
                }

                // Y axis (l<=100)
                {
                    let points: Vec<glam::Vec3> = (0..=100)
                        .map(|i| glam::Vec3::new(0.0, i as f32, 0.0))
                        .collect();
                    self.draw_axis_segment(&painter, points, group_center, Color32::BLACK);
                }

                // Y axis (l<100)
                {
                    let points: Vec<glam::Vec3> = (100..=110)
                        .map(|i| glam::Vec3::new(0.0, i as f32, 0.0))
                        .collect();
                    self.draw_axis_segment(&painter, points, group_center, Color32::GRAY);
                }

                // Z axis (b+)
                {
                    let points: Vec<glam::Vec3> = (-200..=0)
                        .map(|i| glam::Vec3::new(0.0, self.camera_settings.height, i as f32))
                        .collect();
                    self.draw_axis_segment(&painter, points, group_center, Color32::YELLOW);
                }

                // Z axis (b-)
                {
                    let points: Vec<glam::Vec3> = (0..=200)
                        .map(|i| glam::Vec3::new(0.0, self.camera_settings.height, i as f32))
                        .collect();
                    self.draw_axis_segment(&painter, points, group_center, Color32::BLUE);
                }

                self.draw_cube_frame(&painter, group_center);
            });

            ui.add(egui::github_link_file!(
                "https://github.com/Cielquan/GamutPlotty/blob/main/",
                "Source code."
            ));
        });
    }
}
