use color_calc::CIELAB;
use egui::{
    Align, AtomExt, Button, CentralPanel, Color32, ComboBox, Frame, Image, IntoAtoms, Layout,
    MenuBar, Panel, Pos2, Rect, Theme, ThemePreference, emath, epaint, hex_color, include_image,
    pos2,
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
            distance: 1.0,
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

            Frame::canvas(ui.style()).show(ui, |ui| {
                ui.request_repaint();

                let show_colors = true;
                let color = if ui.visuals().dark_mode {
                    Color32::from_additive_luminance(196)
                } else {
                    Color32::from_black_alpha(240)
                };

                let time = ui.input(|i| i.time);

                let desired_size = ui.available_width() * egui::vec2(1.0, 0.35);
                let (_id, rect) = ui.allocate_space(desired_size);

                let to_screen = emath::RectTransform::from_to(
                    Rect::from_x_y_ranges(0.0..=1.0, -1.0..=1.0),
                    rect,
                );

                let mut shapes = vec![];

                for &mode in &[2, 3, 5] {
                    let mode = mode as f64;
                    let n = 120;
                    let speed = 1.5;

                    let points: Vec<Pos2> = (0..=n)
                        .map(|i| {
                            let t = i as f64 / (n as f64);
                            let amp = (time * speed * mode).sin() / mode;
                            let y = amp * (t * std::f64::consts::TAU / 2.0 * mode).sin();
                            to_screen * pos2(t as f32, y as f32)
                        })
                        .collect();

                    let thickness = 10.0 / mode as f32;
                    shapes.push(epaint::Shape::line(
                        points,
                        if show_colors {
                            epaint::PathStroke::new_uv(thickness, move |rect, p| {
                                let t = egui::remap(p.x, rect.x_range(), -1.0..=1.0).abs();
                                let center_color = hex_color!("#5BCEFA");
                                let outer_color = hex_color!("#F5A9B8");

                                Color32::from_rgb(
                                    egui::lerp(center_color.r() as f32..=outer_color.r() as f32, t)
                                        as u8,
                                    egui::lerp(center_color.g() as f32..=outer_color.g() as f32, t)
                                        as u8,
                                    egui::lerp(center_color.b() as f32..=outer_color.b() as f32, t)
                                        as u8,
                                )
                            })
                        } else {
                            epaint::PathStroke::new(thickness, color)
                        },
                    ));
                }

                ui.painter().extend(shapes);
            });

            ui.add(egui::github_link_file!(
                "https://github.com/Cielquan/GamutPlotty/blob/main/",
                "Source code."
            ));
        });
    }
}
