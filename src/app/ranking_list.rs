// use crate::app::download::download::*;

pub mod ranking_list {
    use std::fmt::format;
    use std::time::UNIX_EPOCH;

    use chrono::DateTime;
    use egui::util::id_type_map::TypeId;
    use egui::{load::BytesLoader, Ui};
    use egui::{Color32, Context, RichText};
    use ehttp::Request;
    use json_minimal::Json;
    use poll_promise::Promise;
    use serde::{Deserialize, Serialize};

    use crate::app::data::Data;
    use crate::app::download;
    use crate::app::login::login::{AccessToken, Session};
    use crate::app::rank::{self, RankView};
    use crate::app::schema::schema::{
        Alternative, Criterion, Expert, Ranking, Showable, Variables, Scale,
    };
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
                ranking: Ranking::default(),
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
                                        ret = Some(Box::new(EditRanking::new(
                                            self.ranking.clone(),
                                            &session,
                                            &ctx,
                                            &base_url,
                                        )));
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

    pub struct NewExpert {
        expert: Expert,
        ranking: Ranking,
        error: String,
        download: Download,
    }

    impl NewExpert {
        fn new(ranking: Ranking) -> Self {
            Self {
                expert: Expert::default(),
                ranking,
                error: String::new(),
                download: Download::default(),
            }
        }
    }
    impl View for NewExpert {
        fn show(
            &mut self,
            ui: &mut Ui,
            ctx: &egui::Context,
            base_url: &String,
            session: &Session,
        ) -> Option<Box<dyn View>> {
            if ui.button("Back to ranking editing").clicked() {
                return Some(Box::new(EditRanking::new(
                    self.ranking.clone(),
                    session,
                    ctx,
                    base_url,
                )));
            }
            let mut ret: Option<Box<dyn View>> = None;
            self.expert.show_editable(ui, ctx, base_url, session);

            ui.label(&self.error);
            if ui.button("Submit").clicked() {
                let url = format!("{}/create_expert/{}", base_url, self.ranking.ranking_id);
                if let Err(error) = self.download.post_schema(&self.expert, url, ctx, session) {
                    self.error = error;
                }
            }

            self.download
                .run_when_downloaded(ui, |response, ui| match response.ok {
                    true => {
                        ret = Some(Box::new(EditRanking::new(
                            self.ranking.clone(),
                            session,
                            ctx,
                            base_url,
                        )));
                    }
                    false => match response.text() {
                        Some(err) => {
                            ui.label(err);
                        }
                        None => {
                            ui.label("Unknown error");
                        }
                    },
                });

            ret
        }

        fn get_request(&self, base_url: &String, session: &Session) -> Option<Request> {
            None
        }

        fn populate_from_json(&mut self, json: &String) {
            // Noop
        }
    }

    pub struct NewThing<T> {
        thing: T,
        ranking: Ranking,
        error: String,
        download: Download,
        submit_url_generator: fn(&Ranking, &T, &str) -> String,
    }

    impl<T> NewThing<T>
    where
        T: Default,
    {
        fn new(ranking: Ranking, submit_url_generator: fn(&Ranking, &T, &str) -> String) -> Self {
            Self {
                thing: T::default(),
                ranking,
                error: String::new(),
                download: Download::default(),
                submit_url_generator,
            }
        }
    }

    impl<T> View for NewThing<T>
    where
        T: Serialize + Showable,
    {
        fn show(
            &mut self,
            ui: &mut Ui,
            ctx: &egui::Context,
            base_url: &String,
            session: &Session,
        ) -> Option<Box<dyn View>> {
            if ui.button("Back to ranking editing").clicked() {
                return Some(Box::new(EditRanking::new(
                    self.ranking.clone(),
                    session,
                    ctx,
                    base_url,
                )));
            }
            let mut ret: Option<Box<dyn View>> = None;
            self.thing.show_editable(ui, ctx, base_url, session);
            ui.label(&self.error);

            if ui.button("Submit").clicked() {
                let url = (self.submit_url_generator)(&self.ranking, &self.thing, &base_url);
                if let Err(error) = self.download.post_schema(&self.thing, url, ctx, session) {
                    self.error = error;
                }
            }

            self.download
                .run_when_downloaded(ui, |response, ui| match response.ok {
                    true => {
                        ret = Some(Box::new(EditRanking::new(
                            self.ranking.clone(),
                            session,
                            ctx,
                            base_url,
                        )));
                    }
                    false => match response.text() {
                        Some(err) => {
                            ui.label(err);
                        }
                        None => {
                            ui.label("Unknown error");
                        }
                    },
                });
            ret
        }

        fn get_request(&self, base_url: &String, session: &Session) -> Option<Request> {
            None
        }

        fn populate_from_json(&mut self, json: &String) {
            // noop
        }
    }

    pub struct NewAlternative {
        alternative: Alternative,
        ranking: Ranking,
        error: String,
        download: Download,
    }

    impl NewAlternative {
        fn new(ranking: Ranking) -> Self {
            Self {
                alternative: Alternative::default(),
                ranking,
                error: String::new(),
                download: Download::default(),
            }
        }
    }

    impl View for NewAlternative {
        fn show(
            &mut self,
            ui: &mut Ui,
            ctx: &egui::Context,
            base_url: &String,
            session: &Session,
        ) -> Option<Box<dyn View>> {
            if ui.button("Back to ranking editing").clicked() {
                return Some(Box::new(EditRanking::new(
                    self.ranking.clone(),
                    session,
                    ctx,
                    base_url,
                )));
            }
            let mut ret: Option<Box<dyn View>> = None;
            self.alternative.show_editable(ui, ctx, base_url, session);
            ui.label(&self.error);

            if ui.button("Submit").clicked() {
                let url = format!(
                    "{}/create_alternative/{}",
                    base_url, self.ranking.ranking_id
                );
                if let Err(error) = self
                    .download
                    .post_schema(&self.alternative, url, ctx, session)
                {
                    self.error = error;
                }
            }

            self.download
                .run_when_downloaded(ui, |response, ui| match response.ok {
                    true => {
                        ret = Some(Box::new(EditRanking::new(
                            self.ranking.clone(),
                            session,
                            ctx,
                            base_url,
                        )));
                    }
                    false => match response.text() {
                        Some(err) => {
                            ui.label(err);
                        }
                        None => {
                            ui.label("Unknown error");
                        }
                    },
                });
            ret
        }

        fn get_request(&self, base_url: &String, session: &Session) -> Option<Request> {
            None
        }

        fn populate_from_json(&mut self, json: &String) {
            // No-op
        }
    }

    pub struct EditRanking {
        ranking: Ranking,
        experts_list: Option<Vec<Expert>>,
        alternatives_list: Option<Vec<Alternative>>,
        variables: Option<Variables>,
        criteria: Option<Vec<Criterion>>,
        scale: Option<Vec<Scale>>,
        // TODO:
        // Scale
        error: String,
        download: Download,

        download_variables: Download,
        download_alternatives: Download,
        download_experts: Download,
        download_criteria: Download,
        download_scale: Download,
        // alternative_list: Vec<>
    }

    impl EditRanking {
        fn new(ranking: Ranking, session: &Session, ctx: &Context, base_url: &String) -> Self {
            let mut edit = EditRanking {
                ranking: ranking.clone(),
                experts_list: None,
                alternatives_list: None,
                variables: None,
                criteria: None,
                scale: None,
                error: String::new(),
                download: Download::default(),
                download_variables: Download::default(),
                download_alternatives: Download::default(),
                download_experts: Download::default(),
                download_criteria: Download::default(),
                download_scale: Download::default(),
            };
            let downloader_utility = |middle_url: &str| -> Request {
                let url = format!("{}/{}/{}", &base_url, middle_url, &ranking.ranking_id);
                let request = Request::get(url);
                session.access_token.add_authorization_header(request)
            };
            edit.download_variables
                .download(ctx, downloader_utility("variables"));

            edit.download_alternatives
                .download(ctx, downloader_utility("all_alternatives"));

            edit.download_experts
                .download(ctx, downloader_utility("experts"));

            edit.download_criteria
                .download(ctx, downloader_utility("criteria"));

            edit.download_scale
                .download(ctx, downloader_utility("get_scale"));

            edit
        }
    }
    fn show_section_list<'a, T>(
        id_source: &str,
        ranking: &Ranking,
        ui: &mut Ui,
        ctx: &egui::Context,
        base_url: &String,
        session: &Session,
        value: &mut Option<Vec<T>>,
        download: &mut Download,
        delete_url: fn(&Ranking, &T, &str) -> String,
        create_url: fn(&Ranking, &T, &str) -> String,
        get_url: fn(&Ranking, &str) -> String,
    ) -> Option<Box<dyn View + 'a>>
    where
        T: Default + Serialize + serde::de::DeserializeOwned + Showable + Clone + 'a
    {
        let mut ret: Option<Box<dyn View>> = None;

        if ui.button("Create new").clicked() {
            ret = Some(Box::new(NewThing::<T>::new(ranking.clone(), create_url)));
        }
        if let Some(list) = &value {
            if list.is_empty() {
                ui.label("No items!");
            } else {
                egui::Grid::new(id_source).striped(true).show(ui, |ui| {
                    for value in list {
                        value.show(ui, &ctx, &base_url, &session);
                        if ui.button("Delete item").clicked() {
                            let url = delete_url(&ranking, &value, &base_url);
                            download.delete_schema(url, ctx, session);
                        }
                        ui.end_row();
                    }
                });
            }
            let mut downloaded = false;
            download.run_when_downloaded(ui, |response, ui| match response.ok {
                true => {
                    downloaded = true;
                }
                false => {
                    ui.label(format!(
                        "Got response code {} {}",
                        response.status, response.status_text
                    ));
                }
            });
            if downloaded {
                *value = None;
                download.get_schema(get_url(&ranking, &base_url), ctx, session);
            }
        } else {
            let optional_value: Option<Vec<T>> = download.deserialize_when_got(ui);
            if let Some(v) = optional_value.clone() {
                *value = Some(v);
                download.promise = None;
            }
        }

        ret
    }

    impl View for EditRanking {
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
            ui.separator();
            ui.heading("Basic ranking parameters");
            self.ranking.show_editable(ui, ctx, base_url, session);

            ui.separator();
            ui.heading("Variables");
            egui::Grid::new("Variables grid").show(ui, |ui| {
                if let Some(variables) = &mut self.variables {
                    variables.show(ui);
                    ui.end_row();
                    if ui.button("Create").clicked() {
                        let json = serde_json::to_vec(&variables);
                        match json {
                            Ok(json) => {
                                let url = format!(
                                    "{}/create_variables/{}",
                                    base_url, self.ranking.ranking_id
                                );
                                let mut request = Request::post(url, json);
                                request.headers.insert(
                                    "Content-Type".to_string(),
                                    "application/json".to_string(),
                                );
                                self.download_variables.download(
                                    ctx,
                                    session.access_token.add_authorization_header(request),
                                );
                            }
                            Err(err) => {
                                println!("Failed to serialize variable due to: {}", err);
                            }
                        }
                    }
                    self.download
                        .run_when_downloaded(ui, |response, ui| match response.ok {
                            true => {
                                ui.label("Success");
                                variables.exists_in_ranking = true;
                            }
                            false => match response.text() {
                                Some(err) => {
                                    ui.label(err);
                                }
                                None => {
                                    ui.label("Unknown error");
                                }
                            },
                        });
                } else {
                    self.download_variables
                        .run_when_downloaded(ui, |response, ui| match response.status {
                            200 => match response.text() {
                                Some(json) => {
                                    let variables =
                                        serde_json::from_slice::<Variables>(json.as_bytes());
                                    match variables {
                                        Ok(variables) => {
                                            let mut variables = variables;
                                            variables.exists_in_ranking = true;
                                            self.variables = Some(variables);
                                        }
                                        Err(error) => {
                                            ui.label(format!(
                                                "Failed to parse response due to: {}",
                                                error
                                            ));
                                        }
                                    }
                                }
                                None => {
                                    ui.label("Got empty response from server");
                                }
                            },
                            404 => {
                                if self.variables.is_none() {
                                    self.variables = Some(Variables::default());
                                }
                            }
                            code => {
                                ui.label(format!(
                                    "Failed to get variables, server responded with {} status code",
                                    code
                                ));
                            }
                        });
                }
            });

            ui.separator();
            ui.heading("Alternatives");
            if ui.button("Create new alternative").clicked() {
                ret = Some(Box::new(NewAlternative::new(self.ranking.clone())));
            }
            if let Some(alternatives_list) = &self.alternatives_list {
                if alternatives_list.is_empty() {
                    ui.label("No alternatives!");
                } else {
                    egui::Grid::new("Alternatives")
                        .striped(true)
                        .show(ui, |ui| {
                            for alternative in alternatives_list {
                                alternative.show(ui, &ctx, &base_url, &session);
                                if ui.button("Delete alternative").clicked() {
                                    let url = format!(
                                        "{}/alternative/{}/{}",
                                        base_url,
                                        self.ranking.ranking_id,
                                        alternative.alternative_id
                                    );
                                    self.download_alternatives.delete_schema(url, ctx, session);
                                }
                                ui.end_row();
                            }
                        });
                }
                let mut downloaded = false;
                self.download_alternatives
                    .run_when_downloaded(ui, |response, ui| match response.ok {
                        true => {
                            downloaded = true;
                        }
                        false => {
                            ui.label(format!(
                                "Got response code {} {}",
                                response.status, response.status_text
                            ));
                        }
                    });
                if downloaded {
                    self.alternatives_list = None;
                    self.download_alternatives.get_schema(
                        format!("{}/all_alternatives/{}", base_url, self.ranking.ranking_id),
                        ctx,
                        session,
                    );
                }
            } else {
                if let Some(value) = self.download_alternatives.deserialize_when_got(ui) {
                    self.alternatives_list = Some(value);
                    self.download_alternatives.promise = None;
                }
            }

            ui.separator();
            ui.heading("Experts");

            if ui.button("Create new expert").clicked() {
                ret = Some(Box::new(NewExpert::new(self.ranking.clone())));
            }
            if let Some(experts_list) = &self.experts_list {
                if experts_list.is_empty() {
                    ui.label("No experts!");
                } else {
                    egui::Grid::new("Experts").striped(true).show(ui, |ui| {
                        for expert in experts_list {
                            expert.show(ui, &ctx, &base_url, &session);
                            if ui.button("Delete expert").clicked() {
                                let url = format!(
                                    "{}/experts/{}/{}",
                                    base_url, self.ranking.ranking_id, expert.expert_id
                                );
                                self.download_experts.delete_schema(url, ctx, session);
                            }
                            ui.end_row();
                        }
                    });
                }
                let mut downloaded = false;
                self.download_experts
                    .run_when_downloaded(ui, |response, ui| match response.ok {
                        true => {
                            downloaded = true;
                        }
                        false => {
                            ui.label(format!(
                                "Got response code {} {}",
                                response.status, response.status_text
                            ));
                        }
                    });
                if downloaded {
                    self.experts_list = None;
                    self.download_experts.get_schema(
                        format!("{}/experts/{}", base_url, self.ranking.ranking_id),
                        ctx,
                        session,
                    );
                }
            } else {
                if let Some(value) = self.download_experts.deserialize_when_got(ui) {
                    self.experts_list = Some(value);
                    self.download_experts.promise = None;
                }
            }

            ui.separator();
            ui.heading("Criteria");

            // if ui.button("Create new criterion").clicked() {
            //     ret = Some(Box::new(NewThing::<Criterion>::new(
            //         self.ranking.clone(),
            //         |ranking, _criterion, base_url| {
            //             format!("{}/create_criteria/{}", base_url, ranking.ranking_id)
            //         },
            //     )));
            // }
            // if let Some(list) = &self.criteria {
            //     if list.is_empty() {
            //         ui.label("No criteria!");
            //     } else {
            //         egui::Grid::new("Criteria").striped(true).show(ui, |ui| {
            //             for value in list {
            //                 value.show(ui, &ctx, &base_url, &session);
            //                 if ui.button("Delete criterion").clicked() {
            //                     let url = format!(
            //                         "{}/criteria/{}/{}",
            //                         base_url, self.ranking.ranking_id, value.criteria_id
            //                     );
            //                     self.download_criteria.delete_schema(url, ctx, session);
            //                 }
            //                 ui.end_row();
            //             }
            //         });
            //     }
            //     let mut downloaded = false;
            //     self.download_criteria
            //         .run_when_downloaded(ui, |response, ui| match response.ok {
            //             true => {
            //                 downloaded = true;
            //             }
            //             false => {
            //                 ui.label(format!(
            //                     "Got response code {} {}",
            //                     response.status, response.status_text
            //                 ));
            //             }
            //         });
            //     if downloaded {
            //         self.criteria = None;
            //         self.download_criteria.get_schema(
            //             format!("{}/criteria/{}", base_url, self.ranking.ranking_id),
            //             ctx,
            //             session,
            //         );
            //     }
            // } else {
            //     if let Some(value) = self.download_criteria.deserialize_when_got(ui) {
            //         self.criteria = Some(value);
            //         self.download_criteria.promise = None;
            //     }
            // }

            let out = show_section_list(
                "Criteria",
                &self.ranking,
                ui,
                ctx,
                base_url,
                session,
                &mut self.criteria,
                &mut self.download_criteria,
                |ranking: &Ranking, criterion: &Criterion, base_url: &str| {
                    format!(
                        "{}/criteria/{}/{}",
                        base_url, ranking.ranking_id, criterion.criteria_id
                    )
                },
                |ranking: &Ranking, _: &Criterion, base_url: &str| {
                    format!("{}/create_criteria/{}", base_url, ranking.ranking_id)
                },
                |ranking: &Ranking, base_url: &str| {
                    format!("{}/criteria/{}", base_url, ranking.ranking_id)
                },
            );
            if let Some(v) = out {
                ret = Some(v);
            }

            ui.separator();
            ui.heading("Scale");

            let out = show_section_list(
                "Scale",
                &self.ranking,
                ui,
                ctx,
                base_url,
                session,
                &mut self.scale,
                &mut self.download_scale,
                |ranking: &Ranking, scale: &Scale, base_url: &str| {
                    format!(
                        "{}/scale/{}/{}",
                        base_url, ranking.ranking_id, scale.scale_id
                    )
                },
                |ranking: &Ranking, _: &Scale, base_url: &str| {
                    format!("{}/create_scale/{}", base_url, ranking.ranking_id)
                },
                |ranking: &Ranking, base_url: &str| {
                    format!("{}/get_scale/{}", base_url, ranking.ranking_id)
                },
            );

            if let Some(v) = out {
                ret = Some(v);
            }

            ret
        }

        fn get_request(&self, base_url: &String, session: &Session) -> Option<Request> {
            None
        }

        fn populate_from_json(&mut self, json: &String) {
            // No-op
        }
    }

    pub struct DeleteRanking {
        pub ranking: Ranking,
        download: Download,
    }

    impl DeleteRanking {
        fn new(ranking: Ranking) -> Self {
            Self {
                ranking,
                download: Download::default(),
            }
        }
    }

    impl View for DeleteRanking {
        fn show(
            &mut self,
            ui: &mut Ui,
            ctx: &egui::Context,
            base_url: &String,
            session: &Session,
        ) -> Option<Box<dyn View>> {
            let mut ret: Option<Box<dyn View>> = None;
            ui.vertical_centered_justified(|ui| {
                ui.label(
                    RichText::new(format!(
                        "Are you sure you want to delete ranking '{}'?",
                        self.ranking.description
                    ))
                    .size(20.0),
                );
                ui.columns(2, |columns| {
                    if columns[0].button("Yes, delete").clicked() {
                        let mut request = Request::get(format!(
                            "{}/ranking/{}",
                            base_url, self.ranking.ranking_id
                        ));
                        request.method = "DELETE".to_string();
                        self.download
                            .download(ctx, session.access_token.add_authorization_header(request));
                    }
                    if columns[1].button("No, go back to ranking list").clicked() {
                        ret = Some(Box::new(RankingList::default()));
                    }
                });
            });

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
                            ui.label("Unknown error");
                        }
                    },
                });
            ret
        }

        fn get_request(&self, base_url: &String, session: &Session) -> Option<Request> {
            None
        }

        fn populate_from_json(&mut self, json: &String) {
            // no-op
        }
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
                            ui.label(format!("Id: {}", e.ranking_id));
                        }

                        if session.user_info.admin {
                            if ui.button("Delete").clicked() {
                                ret = Some(Box::new(DeleteRanking::new(e.clone())));
                            }
                            if ui.button("Edit").clicked() {
                                ret = Some(Box::new(EditRanking::new(
                                    e.clone(),
                                    session,
                                    ctx,
                                    base_url,
                                )));
                            }
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
