use egui::{Context, Ui};
use poll_promise::Promise;

use crate::app::view::View;
use json_minimal::{self, Json};
use std::any::Any;

use super::{
    download::download::Download,
    login::login::{LoginForm, Session},
    ranking_list::ranking_list::RankingList,
};

pub struct Data {
    pub base_url: String,
    download: Download,
    current_view: Box<dyn View>,
    parsed: bool,
}

impl Data {
    pub fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:8000".to_string(),
            download: Download::default(),
            current_view: Box::new(RankingList::default()),
            parsed: false,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, ctx: &egui::Context, login: &LoginForm, session: &Session) {
        ui.vertical_centered(|ui| {
            if ui.button("Reload").clicked() {
                self.download.promise = None;
                self.parsed = false;
            }
        });

        if self.parsed {
            match self.current_view.show(ui, ctx, &self.base_url, session) {
                Some(view) => {
                    self.current_view = view;
                    self.parsed = false;
                    self.download.promise = None;
                }
                None => {}
            };
        }

        let request = self.current_view.get_request(&self.base_url, session);

        if let Some(request) = request {
            self.download.download_if_needed(ctx, request);
        } else {
            self.parsed = true;
            return; // Nothing needs to be downloaded
        }

        if !self.parsed {
            if let Some(promise) = &self.download.promise {
                if let Some(result) = promise.ready() {
                    match result {
                        Ok(response) => match response.text() {
                            Some(text) => {
                                self.current_view.populate_from_json(&text.to_string());
                                self.parsed = true;
                            }
                            None => todo!(),
                        },
                        Err(error) => {
                            ui.label(error);
                            self.download.promise = None;
                        }
                    }
                } else {
                    ui.spinner();
                }
            }
        }
    }
}
