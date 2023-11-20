use crate::app::view::View;
use egui::{Response, Ui};
use serde::{Deserialize, Serialize};

use super::data::Data;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Alternative {
    id: i64,
    name: String,
    description: String,
}

impl Alternative {
    fn show(&self, ui: &mut Ui) -> Response {
        return ui
            .vertical_centered(|ui| {
                let ret = ui.button(self.name.clone());
                ui.label(self.description.clone());
                return ret;
            })
            .inner;
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
struct ABInput {
    alternativeA_id: i64,
    alternativeB_id: i64,
    winner_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CriterionInput {
    name: String,
    chosen_option: String,
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
    fn show(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        base_url: &String,
    ) -> Option<Box<dyn View>> {
        let mut ret: Option<Box<dyn View>> = None;
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
                        if let Some(winner) = winner {
                            ret = Some(Box::new(Self::new(self.ranking_id)));
                            let abinput = ABInput {
                                alternativeA_id: abchoice.choiceA.id,
                                alternativeB_id: abchoice.choiceB.id,
                                winner_id: winner,
                            };
                            let mut post = ehttp::Request::post(
                                format!("{}/rankAB/{}", base_url, self.ranking_id),
                                serde_json::to_string(&abinput).unwrap().into_bytes(),
                            );
                            post.headers
                                .insert("Content-Type".to_string(), "application/json".to_string());
                            println!("Submitting {:#?}", post);
                            ehttp::fetch(post, move |result| {
                                if let Err(err) = result {
                                    println!("Failed to post request, but why bother the user?");
                                }
                            });
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
                            if ui.button("Submit").clicked() {
                                ret = Some(Box::new(Self::new(self.ranking_id)));
                                let input = CriterionInput {
                                    chosen_option: criterion_choice.choice_text.clone(),
                                    name: criterion_choice.name.clone(),
                                };

                                let mut post = ehttp::Request::post(
                                    format!("{}/rankCriterion/{}", base_url, self.ranking_id),
                                    serde_json::to_string(&input).unwrap().into_bytes(),
                                );
                                post.headers.insert(
                                    "Content-Type".to_string(),
                                    "application/json".to_string(),
                                );
                                println!("Submitting {:#?}", post);
                                ehttp::fetch(post, move |result| {
                                    if let Err(err) = result {
                                        println!(
                                            "Failed to post request, but why bother the user?"
                                        );
                                    }
                                });
                            }
                        });
                    });
                }
            },
            None => {
                ui.label("Failed to parse response");
            }
        }

        ret
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
