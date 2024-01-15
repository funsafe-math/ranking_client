
pub mod create_ranking {

    #[derive(serde::Deserialize, serde::Serialize)]
    pub struct CreateRanking {
        pub ranking_list: Vec<RankingListItem>,
    }

    impl Default for CreateRanking {
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

                            ret = Some(Box::new(RankView::new(e.id)));
                    }
                }
            });
            egui::Grid::new("ranking_list")
                .striped(true)
                .show(ui, |ui| {
                    for e in &self.ranking_list {
                        let timeout = chrono::DateTime::<chrono::Utc>::UNIX_EPOCH
                            + chrono::Duration::seconds(e.expiring);

                        if ui.button(e.desc.clone()).clicked() {
                            println!("User wants to go to ranking {}", e.id);
                            ret = Some(Box::new(RankView::new(e.id)));
                        }
                        ui.spacing();
                        // TODO: color based on urgency, present in local time
                        ui.label(format!("Expiring: {}", timeout));
                        ui.end_row();
                    }
                });
            ret
        }

        fn get_request(&self, base_url: &String, session: &Session) -> ehttp::Request {
            let rest = match &session.user_info.admin {
                true => "/all_rankings".to_string(),
                false => {
                    format!("/rankings/{}", session.user_info.expert_id)
                }
            };
            let req = ehttp::Request::get(base_url.clone() + &rest);
            session.access_token.add_authorization_header(req)
        }

        fn populate_from_json(&mut self, json: &String) {
            self.ranking_list = serde_json::from_slice(json.as_bytes()).unwrap();
        }
    }
}