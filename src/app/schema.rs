pub mod schema {
    use egui::{Color32, RichText, Ui};
    use ehttp::Request;

    use crate::app::{
        download::download::Download, login::login::Session,
        ranking_list::ranking_list::RankingList, view::View,
    };

    #[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
    pub struct Alternative {
        pub alternative_id: i64,
        pub name: String,
        pub description: String,
    }
    impl Alternative {
        pub fn show(
            &self,
            ui: &mut egui::Ui,
            ctx: &egui::Context,
            base_url: &String,
            session: &Session,
        ) {
            ui.label(&self.name);
            ui.label(&self.description);
            ui.label(self.alternative_id.to_string());
        }

        pub fn show_editable(
            &mut self,
            ui: &mut Ui,
            ctx: &egui::Context,
            base_url: &String,
            session: &Session,
        ) -> Option<Box<dyn View>> {
            egui::Grid::new("Alternative editable")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Name: ");
                    ui.text_edit_singleline(&mut self.name);
                    ui.end_row();
                    ui.label("Description: ");
                    ui.text_edit_singleline(&mut self.description);
                });
            None
        }
    }

    impl Default for Alternative {
        fn default() -> Self {
            Self {
                alternative_id: 0,
                name: "Batman".to_string(),
                description: "Do people still read comic books?".to_string(),
            }
        }
    }

    #[derive(serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, Clone)]
    pub struct Expert {
        pub expert_id: u64,
        pub name: String,
        pub email: String,
        pub admin: bool,
    }

    impl Default for Expert {
        fn default() -> Self {
            Self {
                expert_id: Default::default(),
                name: "Bill Nye".to_string(),
                email: "example@example.com".to_string(),
                admin: false,
            }
        }
    }

    impl Expert {
        pub fn show(
            &self,
            ui: &mut egui::Ui,
            ctx: &egui::Context,
            base_url: &String,
            session: &Session,
        ) {
            ui.label(&self.name);
            ui.label(&self.email);
            ui.label(match self.admin {
                true => "admin".to_string(),
                false => "not admin".to_string(),
            });
            ui.label(self.expert_id.to_string());
        }

        pub fn show_editable(
            &mut self,
            ui: &mut Ui,
            ctx: &egui::Context,
            base_url: &String,
            session: &Session,
        ) -> Option<Box<dyn View>> {
            egui::Grid::new("Expert editable")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Name: ");
                    ui.text_edit_singleline(&mut self.name);
                    ui.end_row();
                    ui.label("email: ");
                    ui.text_edit_singleline(&mut self.email);
                    ui.checkbox(&mut self.admin, "Admin");
                });
            None
        }
    }

    #[derive(serde::Deserialize, serde::Serialize, Clone)]
    pub struct Ranking {
        pub description: String,
        pub ranking_id: i64,
        pub expiring: i64,

        #[serde(skip)]
        download: Download,
    }

    impl Default for Ranking {
        fn default() -> Self {
            Self {
                description: "Superheroes ranking".to_string(),
                ranking_id: Default::default(),
                expiring: Default::default(),
                download: Default::default(),
            }
        }
    }

    impl Ranking {
        pub fn show_editable(
            &mut self,
            ui: &mut Ui,
            ctx: &egui::Context,
            base_url: &String,
            session: &Session,
        ) -> Option<Box<dyn View>> {
            let mut ret: Option<Box<dyn View>> = None;
            egui::Grid::new("Ranking grid")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Description: ");
                    ui.text_edit_singleline(&mut self.description);
                    ui.end_row();

                    ui.label("Expiring: ");
                    //TODO: implement human readable date
                    ui.add(
                        egui::DragValue::new(&mut self.expiring)
                            .speed(1000)
                            .clamp_range(0..=100_000_000),
                    );
                    ui.end_row();

                    if ui.button("Update").clicked() {
                        let json = serde_json::to_vec(&self);
                        match json {
                            Ok(json) => {
                                let url = format!(
                                    "{}/update_ranking_info/{}",
                                    &base_url, &self.ranking_id
                                );
                                let mut request = Request::post(url, json);
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
                                // self.error = error.to_string();
                            }
                        }
                    }

                    self.download
                        .run_when_downloaded(ui, |response, ui| match response.ok {
                            true => {
                                ui.label("Success");
                                ret = Some(Box::new(RankingList::default()));
                            }
                            false => match response.text() {
                                Some(err) => {
                                    ui.label(err);
                                }
                                None => {
                                    ui.label(&response.status_text);
                                }
                            },
                        });
                    ui.end_row();
                });

            ret
        }
    }

    #[derive(serde::Deserialize, serde::Serialize)]
    pub struct Variables {
        pub ranking_method: String,
        pub aggregation_method: String,
        pub completness_required: bool,
        #[serde(skip)]
        pub exists_in_ranking: bool,
        #[serde(skip)]
        download: Download,
    }

    impl Variables {
        pub fn default() -> Self {
            Self {
                ranking_method: "EVM".to_string(),
                aggregation_method: "AIP".to_string(),
                completness_required: true,
                exists_in_ranking: false,
                download: Download::default(),
            }
        }

        pub fn show(&mut self, ui: &mut egui::Ui) {
            egui::Grid::new("Variables grid")
                .num_columns(3)
                .show(ui, |ui| {
                    let mut create_label = |target_str: &mut String, text: &str, ui: &mut Ui| {
                        if ui.selectable_label(target_str == text, text).clicked() {
                            *target_str = text.to_string();
                        }
                    };
                    ui.label("Ranking method:");
                    create_label(&mut self.ranking_method, "EVM", ui);
                    create_label(&mut self.ranking_method, "GMM", ui);
                    ui.end_row();
                    ui.label("Aggregation method:");
                    create_label(&mut self.aggregation_method, "AIJ", ui);
                    create_label(&mut self.aggregation_method, "AIP", ui);

                    ui.end_row();
                    ui.label("Completness required:");
                    ui.checkbox(&mut self.completness_required, "");
                    ui.end_row();

                    if !self.exists_in_ranking {
                        ui.label(
                            RichText::new(
                                "There are no variables in the ranking, consider adding them",
                            )
                            .color(Color32::RED),
                        );
                    }
                });
        }
    }

    #[derive(serde::Deserialize, serde::Serialize, Clone)]
    pub struct Criterion {
        pub criteria_id: i64,
        pub ranking_id: i64,
        pub name: String,
        pub description: String,
    }

    impl Default for Criterion {
        fn default() -> Self {
            Self {
                criteria_id: Default::default(),
                ranking_id: Default::default(),
                name: "Strength".to_string(),
                description: "How strong is the character".to_string(),
            }
        }
    }

    impl Showable for Criterion {
        fn show_editable(
            &mut self,
            ui: &mut Ui,
            ctx: &egui::Context,
            base_url: &String,
            session: &Session,
        ) -> Option<Box<dyn View>> {
            egui::Grid::new("Criterion editable")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Name: ");
                    ui.text_edit_singleline(&mut self.name);
                    ui.end_row();
                    ui.label("description: ");
                    ui.text_edit_singleline(&mut self.description);
                });
            None
        }

        fn show(
            &self,
            ui: &mut egui::Ui,
            ctx: &egui::Context,
            base_url: &String,
            session: &Session,
        ) {
            ui.label(&self.name);
            ui.label(&self.description);
            ui.label(self.criteria_id.to_string());
        }
    }

    pub trait Showable {
        fn show_editable(
            &mut self,
            ui: &mut Ui,
            ctx: &egui::Context,
            base_url: &String,
            session: &Session,
        ) -> Option<Box<dyn View>>;

        fn show(
            &self,
            ui: &mut egui::Ui,
            ctx: &egui::Context,
            base_url: &String,
            session: &Session,
        );
    }

    #[derive(serde::Deserialize, serde::Serialize, Clone)]
    pub struct Scale {
        pub scale_id: i64,
        pub description: String,
        pub value: f32,
        pub ranking_id: i64,
    }

    impl Default for Scale {
        fn default() -> Self {
            Self {
                scale_id: Default::default(),
                description: "Very important".to_string(),
                value: 1.0,
                ranking_id: Default::default(),
            }
        }
    }

    impl Showable for Scale {
        fn show_editable(
            &mut self,
            ui: &mut Ui,
            ctx: &egui::Context,
            base_url: &String,
            session: &Session,
        ) -> Option<Box<dyn View>> {
            egui::Grid::new("Scale editable")
                .num_columns(2)
                .show(ui, |ui| {
                    ui.label("Description: ");
                    ui.text_edit_singleline(&mut self.description);
                    ui.end_row();
                    ui.label("Value: ");
                    ui.add(egui::DragValue::new(&mut self.value).speed(0.1));
                });
            None
        }

        fn show(
            &self,
            ui: &mut egui::Ui,
            ctx: &egui::Context,
            base_url: &String,
            session: &Session,
        ) {
            ui.label(&self.description);
            ui.label(self.value.to_string());
            ui.label(self.scale_id.to_string());
        }
    }
}
