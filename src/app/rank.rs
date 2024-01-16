use std::borrow::Borrow;

use crate::app::view::View;
use combinations::Combinations;
use egui::{Response, Ui};
use ehttp::Request;
use serde::{Deserialize, Serialize};

use super::{
    data::Data,
    download::download::Download,
    login::login::Session,
    ranking_list::ranking_list::RankingList,
    schema::schema::{Alternative, Criterion, Ranking, Scale},
};

// #[derive(Debug, Clone, Serialize, Deserialize)]
// struct Alternative {
//     id: i64,
//     name: String,
//     description: String,
// }

// impl Alternative {
//     fn show(&self, ui: &mut Ui) -> Response {
//         return ui
//             .vertical_centered(|ui| {
//                 let ret = ui.button(self.name.clone());
//                 ui.label(self.description.clone());
//                 return ret;
//             })
//             .inner;
//     }
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// struct ABChoice {
//     choiceA: Alternative,
//     choiceB: Alternative,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// struct CriterionChoice {
//     name: String,
//     description: String,
//     options: Vec<String>,

//     #[serde(skip)]
//     choice_text: String,
// }
// impl Default for CriterionChoice {
//     fn default() -> Self {
//         Self {
//             name: String::new(),
//             description: String::new(),
//             options: Vec::new(),
//             choice_text: String::new(),
//         }
//     }
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// struct ABInput {
//     alternativeA_id: i64,
//     alternativeB_id: i64,
//     winner_id: i64,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// struct CriterionInput {
//     name: String,
//     chosen_option: String,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(tag = "choiceType")]
// enum Choice {
//     ABChoice(ABChoice),
//     CriterionChoice(CriterionChoice),
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ABInput {
    alternativeA_id: i64,
    alternativeB_id: i64,
    winner_id: i64,
    expert_id: u64,
    criteria_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Weights {
    weights_id: i64,
    ranking_id: i64,
    expert_id: u64,
    criteria_id: i64,
    scale_id: i64,
}

enum RankMode {
    Alternative,
    Criterion,
    Finshed,
}

pub struct RankView {
    ranking: Ranking,
    alternatives_list: Option<Vec<Alternative>>,
    criteria: Option<Vec<Criterion>>,
    scale: Option<Vec<Scale>>,
    // TODO:
    // Scale
    error: String,
    download: Download,

    download_alternatives: Download,
    download_criteria: Download,
    download_scale: Download,

    alternative_combinations: Option<Combinations<Alternative>>,
    current_combination_pair: Vec<Alternative>,
    next_combination: bool,
    current_criterion: usize,
    rank_mode: RankMode,
    scale_ix: usize,
}

impl RankView {
    pub fn new(ranking: Ranking, base_url: &str, ctx: &egui::Context, session: &Session) -> Self {
        let mut edit = RankView {
            ranking: ranking.clone(),
            alternatives_list: None,
            criteria: None,
            scale: None,
            error: String::new(),
            download: Download::default(),
            download_alternatives: Download::default(),
            download_criteria: Download::default(),
            download_scale: Download::default(),
            alternative_combinations: None,
            current_combination_pair: Vec::new(),
            next_combination: true,
            current_criterion: 0,
            rank_mode: RankMode::Alternative,
            scale_ix: 0,
        };

        let downloader_utility = |middle_url: &str| -> Request {
            let url = format!("{}/{}/{}", &base_url, middle_url, &ranking.ranking_id);
            let request = Request::get(url);
            session.access_token.add_authorization_header(request)
        };
        edit.download_alternatives
            .download(ctx, downloader_utility("all_alternatives"));

        edit.download_criteria
            .download(ctx, downloader_utility("criteria"));

        edit.download_scale
            .download(ctx, downloader_utility("get_scale"));
        edit
    }
}

impl View for RankView {
    fn show(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        base_url: &String,
        session: &Session,
    ) -> Option<Box<dyn View>> {
        let mut ret: Option<Box<dyn View>> = None;
        if session.user_info.admin {
            if ui.button("Back to ranking_list").clicked() {
                return Some(Box::new(RankingList::default()));
            }
        }
        if let Some(alternatives) = self.download_alternatives.deserialize_when_got(ui) {
            self.alternatives_list = Some(alternatives);
        }

        if let Some(criteria) = self.download_criteria.deserialize_when_got(ui) {
            self.criteria = Some(criteria);
        }

        if let Some(scale) = self.download_scale.deserialize_when_got(ui) {
            self.scale = Some(scale);
        }

        if self.alternatives_list.is_none() || self.criteria.is_none() || self.scale.is_none() {
            ui.spinner();
            return None;
        }
        // Now we know all data is avaiable and can be unwrapped

        let alternatives = self.alternatives_list.as_ref()?;
        let criteria = self.criteria.as_ref().unwrap();
        let scale = self.scale.as_ref().unwrap();

        if alternatives.len() < 3 {
            ui.label(format!(
                "There are only {} alternatives, nothing to compare :(",
                alternatives.len()
            ));
            return None;
        }

        if criteria.len() < 2 {
            ui.label(format!(
                "There are only {} criteria, nothing to compare:)",
                criteria.len()
            ));
            return None;
        }
        if scale.len() < 2 {
            ui.label(format!(
                "There are only {} scales, nothing to compare:)",
                scale.len()
            ));
            return None;
        }

        match self.rank_mode {
            RankMode::Alternative => {
                if self.alternative_combinations.is_none() {
                    self.alternative_combinations =
                        Some(Combinations::new(alternatives.clone(), 2));
                }

                let altenative_combinations = self.alternative_combinations.as_mut().unwrap();

                if self.next_combination {
                    if altenative_combinations.next_combination(&mut self.current_combination_pair)
                    {
                        self.next_combination = false;
                    } else {
                        self.current_criterion += 1;
                        self.alternative_combinations =
                            Some(Combinations::new(alternatives.clone(), 2));
                    }
                }
                if self.current_criterion >= criteria.len() {
                    ui.spinner();
                    ui.label("Not implemented");
                    self.current_criterion = 0;
                    self.rank_mode = RankMode::Criterion;
                    return None;
                    // TODO: go to rank criteria
                }

                let criterion = &criteria[self.current_criterion];

                println!("{:?}", self.current_combination_pair);
                let alternative_a = &self.current_combination_pair[0];
                let alternative_b = &self.current_combination_pair[1];

                ui.vertical_centered(|ui| {
                    ui.heading(format!("Which is better based on: {}", &criterion.name));
                    ui.end_row();
                    ui.spacing();

                    let mut winner_id = None;
                    ui.columns(2, |columns| {
                        if columns[0].button(&alternative_a.name).clicked() {
                            winner_id = Some(alternative_a.alternative_id);
                        }
                        columns[0].end_row();
                        columns[0].label(&alternative_a.description);

                        if columns[1].button(&alternative_b.name).clicked() {
                            winner_id = Some(alternative_b.alternative_id);
                        }
                        columns[1].end_row();
                        columns[1].label(&alternative_b.description);
                    });

                    if let Some(winner_id) = winner_id {
                        let ab_result = ABInput {
                            alternativeA_id: alternative_a.alternative_id,
                            alternativeB_id: alternative_b.alternative_id,
                            winner_id: winner_id,
                            expert_id: session.user_info.expert_id,
                            criteria_id: criterion.criteria_id,
                        };
                        // if self.download.promise.is_none() {
                        if true {
                            ui.label(&self.error);
                            let url = format!("{}/rankAB/{}", base_url, self.ranking.ranking_id);
                            if let Err(error) =
                                self.download.post_schema(&ab_result, url, ctx, session)
                            {
                                self.error = error;
                            }
                        }
                    }
                    let mut got_response = false;
                    self.download
                        .run_when_downloaded(ui, |response, ui| match response.ok {
                            true => {
                                self.next_combination = true;
                                got_response = true;
                            }
                            false => {
                                ui.label(format!(
                                    "Failed, server responed with: {} {}",
                                    response.status, response.status_text
                                ));
                            }
                        });
                    if got_response {
                        self.download.promise = None;
                    }
                });
            }
            RankMode::Criterion => {
                if self.current_criterion >= criteria.len() {
                    self.rank_mode = RankMode::Finshed;
                    return None;
                }
                let criterion = &criteria[self.current_criterion];

                ui.vertical_centered(|ui| {
                    ui.heading(&criterion.name);
                    ui.label(&criterion.description);
                    ui.separator();
                    // println!("{:#?}", criterion_choice);
                    egui::ComboBox::from_label("How important is this criterion?")
                        .selected_text(format!("{}", &scale[self.scale_ix].description))
                        .show_ui(ui, |ui| {
                            for (i, s) in scale.iter().enumerate() {
                                ui.selectable_value(&mut self.scale_ix, i, &s.description);
                            }
                        });
                    ui.separator();
                    if ui.button("Submit").clicked() {
                        let url = format!("{}/weight/{}", base_url, self.ranking.ranking_id);
                        let data = Weights {
                            criteria_id: criterion.criteria_id,
                            weights_id: 0,
                            ranking_id: self.ranking.ranking_id,
                            expert_id: session.user_info.expert_id,
                            scale_id: scale[self.scale_ix].scale_id,
                        };

                        ui.label(&self.error);

                        if let Err(error) = self.download.post_schema(&data, url, ctx, &session) {
                            self.error = error;
                        }
                    }

                    let mut got_response = false;
                    self.download
                        .run_when_downloaded(ui, |response, ui| match response.ok {
                            true => {
                                self.current_criterion += 1;
                                got_response = true;
                            }
                            false => {
                                ui.label(format!(
                                    "Failed, server responed with: {} {}",
                                    response.status, response.status_text
                                ));
                            }
                        });
                    if got_response {
                        self.download.promise = None;
                    }
                });
            }
            RankMode::Finshed => {
                ui.centered_and_justified(|ui| {
                    ui.heading(format!(
                        "Thank you, {} for taking part in our ranking",
                        &session.user_info.name
                    ))
                });
            }
        }

        // match &mut self.choice {
        //     Some(choice) => match choice {
        //         Choice::ABChoice(abchoice) => {
        //             ui.columns(2, |columns| {
        //                 let mut winner = None;
        //                 if abchoice.choiceA.show(&mut columns[0]).clicked() {
        //                     winner = Some(abchoice.choiceA.id);
        //                 }
        //                 if abchoice.choiceB.show(&mut columns[1]).clicked() {
        //                     winner = Some(abchoice.choiceB.id);
        //                 }
        //                 if let Some(winner) = winner {
        //                     ret = Some(Box::new(Self::new(self.ranking_id)));
        //                     let abinput = ABInput {
        //                         alternativeA_id: abchoice.choiceA.id,
        //                         alternativeB_id: abchoice.choiceB.id,
        //                         winner_id: winner,
        //                     };
        //                     let mut post = ehttp::Request::post(
        //                         format!("{}/rankAB/{}", base_url, self.ranking_id),
        //                         serde_json::to_string(&abinput).unwrap().into_bytes(),
        //                     );
        //                     post.headers
        //                         .insert("Content-Type".to_string(), "application/json".to_string());
        //                     println!("Submitting {:#?}", post);
        //                     ehttp::fetch(post, move |result| {
        //                         if let Err(err) = result {
        //                             println!("Failed to post request, but why bother the user?");
        //                         }
        //                     });
        //                 }
        //             });
        //         }
        //         Choice::CriterionChoice(criterion_choice) => {
        //             ui.vertical_centered(|ui| {
        //                 ui.heading(criterion_choice.name.clone());
        //                 ui.label(criterion_choice.description.clone());
        //                 ui.separator();
        //                 // println!("{:#?}", criterion_choice);
        //                 egui::ComboBox::from_label("How important is this criterion?")
        //                     .selected_text(format!("{}", criterion_choice.choice_text))
        //                     .show_ui(ui, |ui| {
        //                         for choice in &criterion_choice.options {
        //                             ui.selectable_value(
        //                                 &mut criterion_choice.choice_text,
        //                                 choice.to_string(),
        //                                 choice,
        //                             );
        //                         }
        //                     });
        //                 ui.separator();
        //                 ui.add_enabled_ui(!criterion_choice.choice_text.is_empty(), |ui| {
        //                     if ui.button("Submit").clicked() {
        //                         ret = Some(Box::new(Self::new(self.ranking_id)));
        //                         let input = CriterionInput {
        //                             chosen_option: criterion_choice.choice_text.clone(),
        //                             name: criterion_choice.name.clone(),
        //                         };

        //                         let mut post = ehttp::Request::post(
        //                             format!("{}/rankCriterion/{}", base_url, self.ranking_id),
        //                             serde_json::to_string(&input).unwrap().into_bytes(),
        //                         );
        //                         post.headers.insert(
        //                             "Content-Type".to_string(),
        //                             "application/json".to_string(),
        //                         );
        //                         println!("Submitting {:#?}", post);
        //                         ehttp::fetch(post, move |result| {
        //                             if let Err(err) = result {
        //                                 println!(
        //                                     "Failed to post request, but why bother the user?"
        //                                 );
        //                             }
        //                         });
        //                     }
        //                 });
        //             });
        //         }
        //     },
        //     None => {
        //         ui.label("Failed to parse response");
        //     }
        // }

        ret
    }

    fn get_request(&self, base_url: &String, session: &Session) -> Option<ehttp::Request> {
        // Some(ehttp::Request::get(format!(
        //     "{}/rank/{}",
        //     base_url, self.ranking_id
        // )))
        None
    }

    fn populate_from_json(&mut self, json: &String) {
        // let choice: Result<Choice, serde_json::Error> = serde_json::from_slice(json.as_bytes());
        // match choice {
        //     Ok(choice) => {
        //         self.choice = Some(choice);
        //     }
        //     Err(err) => {
        //         println!("Failed to parse choice, error: {}", err);
        //     }
        // }
        // self.choice = Some(serde_json::from_slice(json.as_bytes()).unwrap());
    }
}
