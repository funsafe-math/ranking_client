pub mod schema {
    #[derive(serde::Deserialize, serde::Serialize)]
    pub struct Alternative {
        pub alternative_id: i64,
        pub name: String,
        pub description: String,
    }

    #[derive(serde::Deserialize, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
    pub struct Expert {
        pub expert_id: u64,
        pub name: String,
        pub email: String,
        pub admin: bool,
    }

    #[derive(serde::Deserialize, serde::Serialize)]
    pub struct Ranking {
        pub description: String,
        pub ranking_id: i64,
        pub expiring: i64,
    }
}
