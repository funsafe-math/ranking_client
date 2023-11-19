// use crate::app::download::download::*;

pub mod ranking_list {
    use egui::Ui;
    use poll_promise::Promise;

    use crate::app::download::download::Download;

    #[derive(serde::Deserialize, serde::Serialize)]
    pub struct RankingListItem {
        pub desc: String,
        pub id: u32,
        pub expiring: chrono::DateTime<chrono::Utc>,
    }

    #[derive(serde::Deserialize, serde::Serialize)]
    pub struct RankingList {
        pub ranking_list: Vec<RankingListItem>,
        // downloaded: Arc<Mutex<bool>>,
        #[serde(skip)]
        // pub promise: Option<Promise<ehttp::Response>>,
        pub download: Download,
    }

    impl Default for RankingList {
        fn default() -> Self {
            Self {
                ranking_list: Vec::new(),
                download: Download::new("/rankings".to_string()),
            }
        }
    }

    impl RankingList {
        pub fn show(&mut self, ui: &mut Ui, ctx: &egui::Context) {
            self.download.download_if_needed(ctx);

            ui.heading("Available rankings");

            if let Some(response) = self.download.downloaded() {
                ui.label(response.text().unwrap());
            } else {
                ui.spinner();
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                // Add a lot of widgets here.
                if ui.button("Click me").clicked() {
                    // take some action here
                }
            });
        }
    }
}
