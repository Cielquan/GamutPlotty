use color_calc::CIELAB;
use egui::{AtomExt, Button, IntoAtoms};

use crate::dummy_state::create_color_points;

const APP_NAME: &str = "Gamut Plotty";
const APP_KEY: &str = "gamut_plotty_app";

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct GamutPlottyApp {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
    color_points: Vec<CIELAB::LabPoint>,
}

impl Default for GamutPlottyApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: APP_NAME.to_owned(),
            value: 2.7,
            color_points: create_color_points(),
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

        egui::Panel::top("top_panel").show_inside(ui, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
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
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.horizontal(|ui| {
                        let current_system_theme =
                            if let Some(system_theme) = ui.input(|i| i.raw.system_theme) {
                                match system_theme {
                                    egui::Theme::Dark => "dark",
                                    egui::Theme::Light => "light",
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
                            egui::ThemePreference::Light,
                            (
                                egui::Image::new(egui::include_image!(
                                    "../../assets/images/sun.svg"
                                ))
                                .atom_max_height_font_size(ui),
                                "Light",
                            ),
                        )
                        .on_hover_text("Use light mode");

                        theme_button(
                            ui,
                            &mut theme_preference,
                            egui::ThemePreference::Dark,
                            (
                                egui::Image::new(egui::include_image!(
                                    "../../assets/images/moon.svg"
                                )),
                                "Dark",
                            ),
                        )
                        .on_hover_text("Use dark mode");

                        theme_button(
                            ui,
                            &mut theme_preference,
                            egui::ThemePreference::System,
                            (
                                egui::Image::new(egui::include_image!(
                                    "../../assets/images/sun-moon.svg"
                                )),
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

        egui::CentralPanel::default().show_inside(ui, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading(APP_NAME);

            ui.horizontal(|ui| {
                ui.label("Write something: ");
                ui.text_edit_singleline(&mut self.label);
            });

            ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            if ui.button("Increment").clicked() {
                self.value += 1.0;
            }

            ui.separator();

            ui.add(egui::github_link_file!(
                "https://github.com/Cielquan/GamutPlotty/blob/main/",
                "Source code."
            ));
        });
    }
}
