// use crate::app::download::download::*;

pub mod ranking_list {

    use std::time::UNIX_EPOCH;

    use chrono::DateTime;
    use egui::{Ui, load::BytesLoader};
    use json_minimal::Json;
    use poll_promise::Promise;

    use crate::app::{download::download::Download, view::View};
    use crate::app::rank::{self, RankView};

    #[derive(serde::Deserialize, serde::Serialize)]
    pub struct RankingListItem {
        pub desc: String,
        pub id: i64,
        pub expiring: i64,
    }

    #[derive(serde::Deserialize, serde::Serialize)]
    pub struct RankingList {
        pub ranking_list: Vec<RankingListItem>,
    }

    impl Default for RankingList {
        fn default() -> Self {
            Self {
                ranking_list: Vec::new(),
            }
        }
    }

    impl View for RankingList {
        fn show(&mut self, ui: &mut Ui, ctx: &egui::Context) -> std::option::Option<Box<(dyn View + 'static)>> {
            let mut ret: Option<Box<dyn View>> = None;
            ui.heading("Available rankings");
            egui::ScrollArea::vertical().show(ui, |ui| {
                // Add a lot of widgets here.
                for e in &self.ranking_list {
                    ui.horizontal(|ui| {
                        // ui.label(e.desc.clone());
                        let timeout = chrono::DateTime::<chrono::Utc>::UNIX_EPOCH + chrono::Duration::seconds(e.expiring);

                        if ui.button(e.desc.clone()).clicked() {
                            println!("User wants to go to ranking {}", e.id);
                            ret = Some(Box::new(RankView::new(e.id)));
                            // TODO: implement loading other page
                        }
                        ui.spacing();
                        // TODO: color based on urgency, present in local time
                        ui.label(format!("Expiring: {}", timeout));
                    });
                }
            });
            ret
        }

        fn get_request(&self, base_url: &String) -> ehttp::Request {
            ehttp::Request::get(base_url.clone() + "/rankings")
        }

        fn populate_from_json(&mut self, json: &String) {
            self.ranking_list = serde_json::from_slice(json.as_bytes()).unwrap();
        }
    }
}
