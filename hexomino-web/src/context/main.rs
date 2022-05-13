use api::{RoomId, GameId};
use yew::Callback;

use crate::view::{MainMsg, Route};

pub struct MainLink {
    main_callback: Callback<MainMsg>,
}

impl MainLink {
    pub fn new(main_callback: Callback<MainMsg>) -> Self {
        Self {
            main_callback
        }
    }

    pub fn go(&self, route: Route) {
        self.main_callback.emit(MainMsg::OnRouteChange(route));
    }
}
