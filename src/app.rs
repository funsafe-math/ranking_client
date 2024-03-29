use std::any::Any;

use egui::{Context, FontId, RichText};
use poll_promise::Promise;

mod data;
mod download;
mod login;
mod rank;
mod ranking_list;
mod view;
mod schema;

use login::login::*;
use ranking_list::ranking_list::*;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,

    #[serde(skip)]
    data: data::Data,

    #[serde(skip)]
    login_form: LoginForm,
}

impl Default for TemplateApp {
    fn default() -> Self {
        Self {
            value: 2.7,
            data: data::Data::default(),
            login_form: LoginForm::default(),
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }
}

impl eframe::App for TemplateApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
                // {
                //     ui.menu_button("File", |ui| {
                //         if ui.button("Quit").clicked() {
                //             _frame.close();
                //         }
                //     });
                //     ui.add_space(16.0);
                // }

                egui::widgets::global_dark_light_mode_buttons(ui);
                ui.add_space(15.0);
                if let LoginStep::Finished(session) = &self.login_form.step {
                    if session.user_info.admin {
                        ui.label("Logged in as admin: ");
                    } else {
                        ui.label("Logged in as: ");
                    }
                }
                ui.monospace(&self.login_form.email);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            if let LoginStep::Finished(session) = &self.login_form.step {
                self.data.show(ui, ctx, &self.login_form, session);
            } else {
                self.login_form.show(ui, ctx, &self.data);
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                // powered_by_egui_and_eframe(ui);
                egui::warn_if_debug_build(ui);
            });
        });
    }
}
