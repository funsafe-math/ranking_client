// use crate::app::download::download::*;

pub mod ranking_list {
    use std::fmt::format;
    use std::time::UNIX_EPOCH;

    use chrono::DateTime;
    use egui::{load::BytesLoader, Ui};
    use ehttp::Request;
    use json_minimal::Json;
    use poll_promise::Promise;
    use serde::Serialize;

    use crate::app::data::Data;
    use crate::app::login::login::{AccessToken, Session};
    use crate::app::rank::{self, RankView};
    use crate::app::schema::schema::{Expert, Ranking};
    use crate::app::{download::download::Download, view::View};

    pub struct NewRanking {
        pub ranking: Ranking,
        pub expiring_date: chrono::naive::NaiveDate,
        pub download: Download,
        pub error: String,
    }

    impl Default for NewRanking {
        fn default() -> Self {
            Self {
                ranking: Ranking {
                    description: "Superheroes ranking".to_string(),
                    ranking_id: 0,
                    expiring: 0,
                },
                expiring_date: chrono::NaiveDate::default(),
                download: Download::default(),
                error: String::new(),
            }
        }
    }
    impl View for NewRanking {
        fn show(
            &mut self,
            ui: &mut Ui,
            ctx: &egui::Context,
            base_url: &String,
            session: &Session,
        ) -> Option<Box<dyn View>> {
            if ui.button("Back to ranking_list").clicked() {
                return Some(Box::new(RankingList::default()));
            }
            let mut ret: Option<Box<dyn View>> = None;
            egui::Grid::new("New ranking grid")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Description: ");
                    ui.text_edit_singleline(&mut self.ranking.description);
                    ui.end_row();

                    ui.label("Expiring: ");
                    //TODO: implement human readable date
                    ui.add(
                        egui::DragValue::new(&mut self.ranking.expiring)
                            .speed(1000)
                            .clamp_range(0..=100_000_000),
                    );
                    // ui.(&mut self.ranking.desc);
                    // egui_ex
                    ui.end_row();

                    ui.label(&self.error);
                    if ui.button("Submit").clicked() {
                        let json = serde_json::to_vec(&self.ranking);
                        match json {
                            Ok(json) => {
                                let mut request =
                                    Request::post(base_url.clone() + "/create_ranking", json);
                                request.headers.insert(
                                    "Content-Type".to_string(),
                                    "application/json".to_string(),
                                );
                                self.download.download(
                                    ctx,
                                    session.access_token.add_authorization_header(request),
                                );
                            }
                            Err(error) => {
                                self.error = error.to_string();
                            }
                        }
                    }

                    if let Some(promise) = &self.download.promise {
                        if let Some(result) = promise.ready() {
                            match result {
                                Ok(response) => match response.ok {
                                    true => {
                                        ui.label("Success");
                                        ret = Some(Box::new(RankingList::default()));
                                    }
                                    false => match response.text() {
                                        Some(err) => {
                                            ui.label(err);
                                        }
                                        None => {
                                            ui.label("Unknown error");
                                        }
                                    },
                                },
                                Err(error) => {
                                    self.error = error.clone();
                                    ui.label(error.clone());
                                    self.download.promise = None;
                                }
                            }
                        } else {
                            ui.spinner();
                        }
                    }
                    ui.end_row();
                });
            ret
        }

        fn get_request(&self, base_url: &String, session: &Session) -> Option<ehttp::Request> {
            None
        }

        fn populate_from_json(&mut self, json: &String) {
            // No-op
        }
    }

    #[derive(serde::Deserialize, serde::Serialize)]
    pub struct EditRanking {
        ranking: Ranking,
        experts_list: Vec<Expert>,
        // alternative_list: Vec<>
    }

    #[derive(serde::Deserialize, serde::Serialize)]
    pub struct RankingList {
        pub ranking_list: Vec<Ranking>,
    }

    impl Default for RankingList {
        fn default() -> Self {
            Self {
                ranking_list: Vec::new(),
            }
        }
    }

    impl View for RankingList {
        fn show(
            &mut self,
            ui: &mut Ui,
            ctx: &egui::Context,
            base_url: &std::string::String,
            session: &Session,
        ) -> std::option::Option<Box<dyn View>> {
            let mut ret: Option<Box<dyn View>> = None;
            ui.horizontal(|ui| {
                ui.heading("Available rankings");
                if session.user_info.admin {
                    if ui.button("Create new ranking").clicked() {
                        ret = Some(Box::new(NewRanking::default()));
                    }
                }
            });
            egui::Grid::new("ranking_list")
                .striped(true)
                .show(ui, |ui| {
                    for e in &self.ranking_list {
                        let timeout = chrono::DateTime::<chrono::Utc>::UNIX_EPOCH
                            + chrono::Duration::seconds(e.expiring);

                        if ui.button(e.description.clone()).clicked() {
                            println!("User wants to go to ranking {}", e.ranking_id);
                            ret = Some(Box::new(RankView::new(e.ranking_id)));
                        }
                        ui.spacing();
                        // TODO: color based on urgency, present in local time
                        ui.label(format!("Expiring: {}", timeout));

                        if session.user_info.admin {
                            ui.button("Delete (not implemented)"); //TODO: implement
                            ui.button("Edit (not implemented)"); //TODO: implement
                        }
                        ui.end_row();
                    }
                });
            ret
        }

        fn get_request(&self, base_url: &String, session: &Session) -> Option<ehttp::Request> {
            let rest = match &session.user_info.admin {
                true => "/all_rankings".to_string(),
                false => {
                    format!("/rankings/{}", session.user_info.expert_id)
                }
            };
            let req = ehttp::Request::get(base_url.clone() + &rest);
            Some(session.access_token.add_authorization_header(req))
        }

        fn populate_from_json(&mut self, json: &String) {
            let ranking_list: Result<Vec<Ranking>, serde_json::Error> =
                serde_json::from_slice(json.as_bytes());
            match ranking_list {
                Ok(ranking_list) => self.ranking_list = ranking_list,
                Err(err) => {
                    println!("Failed to parse ranking_list, error: {}", err);
                }
            }
        }
    }
}
