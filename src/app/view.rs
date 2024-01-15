use egui::Ui;
use ehttp::Request;
use json_minimal::Json;

use crate::app::data::Data;

use super::login::login::{AccessToken, Session};

pub trait View {
    fn show(
        &mut self,
        ui: &mut Ui,
        ctx: &egui::Context,
        base_url: &String,
        session: &Session,
    ) -> Option<Box<dyn View>>;
    fn get_request(&self, base_url: &String, session: &Session) -> Option<Request>;
    fn populate_from_json(&mut self, json: &String);
}
