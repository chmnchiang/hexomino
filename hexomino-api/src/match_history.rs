use chrono::{DateTime, Utc};

use crate::{derive_api_data, Api, MatchId, MatchConfig, MatchToken};

derive_api_data! {
    pub struct MatchHistoryNoGames {
        pub id: MatchId,
        pub users: [String; 2],
        pub user_is_first: bool,
        pub scores: [u32; 2],
        pub end_time: DateTime<Utc>,
        pub config: Option<MatchConfig>,
        pub match_token: Option<MatchToken>,
    }
}

derive_api_data! {
    pub struct ListUserMatchHistoriesApi;
}

pub type ListUserMatchHistoriesApiRequest = ();
pub type ListUserMatchHistoriesApiResponse = Vec<MatchHistoryNoGames>;
impl Api for ListUserMatchHistoriesApi {
    type Request = ListUserMatchHistoriesApiRequest;
    type Response = ListUserMatchHistoriesApiResponse;
}
