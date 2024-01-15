pub mod login {

    use crate::app::{
        data::Data,
        download::{self, download::Download}, schema::schema::Expert,
    };
    use egui::{Response, Ui};
    use ehttp::Request;
    use serde::Deserialize;

    #[derive(serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, Clone)]
    pub struct AccessToken {
        acces_token: String,
        token_type: String,
    }

    impl AccessToken {
        pub fn add_authorization_header(&self, request: Request) -> Request {
            let mut request = request;
            request.headers.insert(
                "Authorization".to_string(),
                format!("{} {}", self.token_type, self.acces_token),
            );
            request
        }
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    pub struct Session {
        pub access_token: AccessToken,
        pub user_info: Expert,
    }

    impl Session {
        fn new(access_token: AccessToken, expert: Expert) -> Self {
            Self {
                access_token: access_token,
                user_info: expert,
            }
        }
    }

    #[derive(PartialEq, Eq, PartialOrd, Ord)]
    pub enum LoginStep {
        Authentication,
        GetUserInfo(AccessToken),
        Finished(Session),
    }

    pub struct LoginForm {
        pub email: String,
        pub download: Download,
        error: String,
        pub step: LoginStep,
    }

    impl Default for LoginForm {
        fn default() -> Self {
            Self {
                email: "example@example.com".to_string(),
                download: Default::default(),
                error: "".to_string(),
                step: LoginStep::Authentication,
            }
        }
    }

    impl LoginForm {
        pub fn show(&mut self, ui: &mut Ui, ctx: &egui::Context, data: &Data) {
            // ui.horizontal_centered(|ui| {
            egui::Grid::new("Login form").num_columns(2).show(ui, |ui| {
                ui.label("Email: ");
                ui.text_edit_singleline(&mut self.email);
                ui.end_row();

                if ui.button("Login").clicked() {
                    let mut request = Request::post(
                        data.base_url.clone() + "/token",
                        format!("grant_type=password&username={}&password=abc", self.email)
                            .to_string()
                            .as_bytes()
                            .to_vec(),
                    );
                    request.headers.insert(
                        "Content-Type".to_string(),
                        "application/x-www-form-urlencoded".to_string(),
                    );
                    self.download.download(ctx, request);
                }

                if let Some(promise) = &self.download.promise {
                    if let Some(result) = promise.ready() {
                        match result {
                            Ok(response) => match response.text() {
                                Some(json) => {
                                    match &self.step {
                                        LoginStep::Authentication => {
                                            if response.ok {
                                                if let Ok(token) =
                                                    serde_json::from_str::<AccessToken>(json)
                                                {
                                                    let request =
                                                        Request::get(data.base_url.clone() + "/me");
                                                    let request =
                                                        token.add_authorization_header(request);
                                                    self.step = LoginStep::GetUserInfo(token);
                                                    self.download.download(ctx, request);
                                                    return;
                                                } else {
                                                    ui.label("Failed to deserialze token response");
                                                }
                                            } else {
                                                ui.label(format!(
                                                    "Failed to log in due to: {}",
                                                    &response.status_text
                                                ));
                                            }
                                        }
                                        LoginStep::GetUserInfo(token) => {
                                            if response.ok {
                                                if let Ok(expert) =
                                                    serde_json::from_str::<Expert>(json)
                                                {
                                                    println!(
                                                        "Logged in as admin? {}",
                                                        expert.admin
                                                    );
                                                    self.step = LoginStep::Finished(Session::new(
                                                        token.clone(),
                                                        expert,
                                                    ));
                                                } else {
                                                    ui.label("Failed to deserialze token response");
                                                }
                                            } else {
                                                ui.label(format!(
                                                    "Failed to log in due to: {}",
                                                    &response.status_text
                                                ));
                                            }
                                        }

                                        LoginStep::Finished(session) => {}
                                    }
                                    println!("{}", &response.text().unwrap());
                                }
                                None => {
                                    ui.label("Failed request");
                                }
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
            // });

            // ui.horizontal_centered(|ui| {
            // });
            // ui.label("test");
        }
    }
}
