pub mod download {
    use egui::{Context, Response};
    use ehttp::Request;
    use poll_promise::Promise;
    use serde::Deserialize;

    use crate::app::login::login::{LoginForm, Session};

    pub struct Download {
        pub promise: Option<Promise<Result<ehttp::Response, String>>>,
        path: String,
    }

    impl Default for Download {
        fn default() -> Self {
            Download {
                promise: None,
                path: "/me".to_string(),
            }
        }
    }

    impl Clone for Download {
        fn clone(&self) -> Self {
            Self {
                promise: None,
                path: self.path.clone(),
            }
        }
    }

    impl Download {
        pub fn new(path: String) -> Self {
            Self {
                promise: None,
                path: path,
            }
        }

        pub fn post_schema<T>(
            &mut self,
            value: &T,
            url: String,
            ctx: &Context,
            session: &Session,
        ) -> Result<bool, String>
        where
            T: serde::Serialize,
        {
            let json = serde_json::to_vec(value);
            match json {
                Ok(json) => {
                    let mut request = Request::post(url, json);
                    request
                        .headers
                        .insert("Content-Type".to_string(), "application/json".to_string());
                    self.download(ctx, session.access_token.add_authorization_header(request));
                    Result::Ok(true)
                }
                Err(error) => {
                    // self.error = error.to_string();
                    Result::Err(error.to_string())
                }
            }
        }

        pub fn delete_schema(&mut self, url: String, ctx: &Context, session: &Session) {
            let mut request = Request::post(url, "".to_string().into_bytes());
            request.method = "DELETE".to_string();
            self.download(ctx, session.access_token.add_authorization_header(request));
        }

        pub fn get_schema(&mut self, url: String, ctx: &Context, session: &Session){
            let mut request = Request::get(url);
            self.download(ctx, session.access_token.add_authorization_header(request));

        }

        pub fn deserialize_when_got<T>(&self, ui: &mut egui::Ui) -> Option<T>
        where
            T: serde::de::DeserializeOwned
        {
            if let Some(promise) = &self.promise {
                if let Some(result) = promise.ready() {
                    match result {
                        Ok(response) => match response.ok {
                            true => match response.text() {
                                Some(json) => {
                                    let value = serde_json::from_str::<T>(json);
                                    match value {
                                        Ok(value) => {
                                            return Some(value);
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
                                    ui.label("Empty response from server");
                                }
                            },
                            false => {
                                ui.label(format!(
                                    "Server responded with {}",
                                    &response.status_text
                                ));
                            }
                        },
                        Err(error) => {
                            ui.label(error.clone());
                        }
                    }
                } else {
                    ui.spinner();
                }
            }
            None
        }

        pub fn run_when_downloaded<F>(&self, ui: &mut egui::Ui, func: F)
        where
            F: FnOnce(&ehttp::Response, &mut egui::Ui),
        {
            if let Some(promise) = &self.promise {
                if let Some(result) = promise.ready() {
                    match result {
                        Ok(response) => func(&response, ui),
                        Err(error) => {
                            ui.label(error.clone());
                        }
                    }
                } else {
                    ui.spinner();
                }
            }
        }

        pub fn download_if_needed(&mut self, ctx: &Context, request: Request) {
            if self.promise.is_none() {
                let (sender, promise) = Promise::new();
                self.promise = Some(promise);
                let mut request = request.clone();
                request
                    .headers
                    .insert("Access-Control-Allow-Origin".to_string(), "*".to_string());
                let ctx = ctx.clone();
                println!("Fetching {}", request.url);
                ehttp::fetch(request, move |response: Result<ehttp::Response, String>| {
                    ctx.request_repaint();
                    println!("Got response : {}", response.is_ok());
                    sender.send(response);
                });
            }
        }
        pub fn download(&mut self, ctx: &Context, request: Request) {
            // match &self.promise {
            //     Some(x) => {x.ready().clone()},
            //     None => {None},
            // }
            // todo!();
            self.promise = None;
            self.download_if_needed(ctx, request);
        }
    }
}
