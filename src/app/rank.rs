use crate::app::view::View;
use egui::{Ui, Response};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Alternative {
    id: i64,
    name: String,
    description: String,
}

impl Alternative {
    fn show(&self, ui: &mut Ui) -> Response {
        let mut ret = None;
        ui.vertical_centered(|ui| {
            ret = Some(ui.button(self.name.clone()));
            ui.label(self.description.clone());
        });
        return ret.unwrap();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ABChoice {
    choiceA: Alternative,
    choiceB: Alternative,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CriterionChoice {
    name: String,
    description: String,
    options: Vec<String>,

    #[serde(skip)]
    choice_text: String,
}
impl Default for CriterionChoice {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            options: Vec::new(),
            choice_text: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "choiceType")]
enum Choice {
    ABChoice(ABChoice),
    CriterionChoice(CriterionChoice),
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct RankView {
    choice: Option<Choice>,
    ranking_id: i64,
}

impl RankView {
    pub fn new(ranking_id: i64) -> Self {
        Self {
            choice: None,
            ranking_id: ranking_id,
        }
    }
}

impl View for RankView {
    fn show(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) -> Option<Box<dyn View>> {
        match &mut self.choice {
            Some(choice) => match choice {
                Choice::ABChoice(abchoice) => {
                    ui.columns(2, |columns| {
                        let mut winner = None;
                        if abchoice.choiceA.show(&mut columns[0]).clicked() {
                            winner = Some(abchoice.choiceA.id);
                        }
                        if abchoice.choiceB.show(&mut columns[1]).clicked() {
                            winner = Some(abchoice.choiceB.id);
                        }
                    });
                }
                Choice::CriterionChoice(criterion_choice) => {
                    ui.vertical_centered(|ui| {
                        ui.heading(criterion_choice.name.clone());
                        ui.label(criterion_choice.description.clone());
                        ui.separator();
                        // println!("{:#?}", criterion_choice);
                        egui::ComboBox::from_label("How important is this criterion?")
                            .selected_text(format!("{}", criterion_choice.choice_text))
                            .show_ui(ui, |ui| {
                                for choice in &criterion_choice.options {
                                    ui.selectable_value(
                                        &mut criterion_choice.choice_text,
                                        choice.to_string(),
                                        choice,
                                    );
                                }
                            });
                        ui.separator();
                        ui.add_enabled_ui(!criterion_choice.choice_text.is_empty(), |ui| {
                            ui.button("Submit");
                        });
                    });
                }
            },
            None => {
                ui.label("Failed to parse response");
            }
        }

        None
    }

    fn get_request(&self, base_url: &String) -> ehttp::Request {
        ehttp::Request::get(format!("{}/rank/{}", base_url, self.ranking_id))
    }

    fn populate_from_json(&mut self, json: &String) {
        let choice: Result<Choice, serde_json::Error> = serde_json::from_slice(json.as_bytes());
        match choice {
            Ok(choice) => {
                self.choice = Some(choice);
            }
            Err(err) => {
                println!("Failed to parse choice, error: {}", err);
            }
        }
        // self.choice = Some(serde_json::from_slice(json.as_bytes()).unwrap());
    }
}
