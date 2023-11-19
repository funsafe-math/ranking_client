use egui::Ui;
use ehttp::Request;
use json_minimal::Json;

pub trait View {
    fn show(&mut self, ui: &mut Ui, ctx: &egui::Context) -> Option<Box<dyn View>>;
    fn get_request(&self, base_url: &String) -> Request;
    fn populate_from_json(&mut self, json: &String);
}