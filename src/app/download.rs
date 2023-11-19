pub mod download {
    use egui::Context;
    use ehttp::Request;
    use poll_promise::Promise;

pub struct Download {
    pub promise: Option<Promise<Result<ehttp::Response, String>>>,
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

    pub fn download_if_needed(&mut self, ctx: &Context, request: Request) {
        if self.promise.is_none() {
            let (sender, promise) = Promise::new();
            self.promise = Some(promise);
            let mut request = request.clone();
            request.headers.insert("Access-Control-Allow-Origin".to_string(), "*".to_string());
            let ctx = ctx.clone();
            println!("Fetching {}", request.url);
            ehttp::fetch(request, move |response: Result<ehttp::Response, String>| {
                ctx.request_repaint();
                println!("Got response : {}", response.is_ok());
                sender.send(response);
            });
        }
    }
    pub fn downloaded(&self) -> Option<&ehttp::Response> {
        // match &self.promise {
        //     Some(x) => {x.ready().clone()},
        //     None => {None},
        // }
        todo!();
    }
}
}