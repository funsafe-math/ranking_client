use egui::{Ui};


use crate::app::view::View;
use json_minimal::{self};


use super::{download::download::Download, ranking_list::ranking_list::RankingList};

pub struct Data {
    base_url: String,
    download: Download,
    current_view: Box<dyn View>,
    parsed: bool,
}

impl Data {
    pub fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:9999".to_string(),
            download: Download::default(),
            current_view: Box::new(RankingList::default()),
            parsed: false,
        }
    }

    pub fn show(&mut self, ui: &mut Ui, ctx: &egui::Context) {
        ui.vertical_centered(|ui| {
            if ui.button("Reload").clicked() {
                self.download.promise = None;
                self.parsed = false;
            }
        });

        if self.parsed {
            match self.current_view.show(ui, ctx, &self.base_url) {
                Some(view) => {
                    self.current_view = view;
                    self.parsed = false;
                    self.download.promise = None;
                }
                None => {}
            };
        }

        self.download
            .download_if_needed(ctx, self.current_view.get_request(&self.base_url));

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
