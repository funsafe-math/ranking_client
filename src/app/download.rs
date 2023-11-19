pub mod download {
    use egui::Context;
    use poll_promise::Promise;

pub struct Download {
    pub promise: Option<Promise<ehttp::Response>>,
    path: String,
}

impl Default for Download {
    fn default() -> Self {
        Download { promise: None, path: "/rankings".to_string() }
    }
}

impl Download {
    pub fn new(path: String)-> Self{
        Self{promise: None, path: path}
    }

    pub fn download_if_needed(&mut self, ctx: &Context) {
        if self.promise.is_none() {
            let (sender, promise) = Promise::new();
            self.promise = Some(promise);
            let url = "http://127.0.0.1:8000/rankings";
            let request = ehttp::Request::get(url);
            let ctx = ctx.clone();
            ehttp::fetch(request, move |response: Result<ehttp::Response, String>| {
                ctx.request_repaint();
                println!("Got response : {}", response.is_ok());
                sender.send(response.unwrap());
            });
        }
    }
    pub fn downloaded(&self) -> Option<&ehttp::Response> {
        match &self.promise {
            Some(x) => {x.ready().clone()},
            None => {None},
        }
    }
}
}