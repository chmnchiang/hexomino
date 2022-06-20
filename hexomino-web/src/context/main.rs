use std::{cell::RefCell, fmt::Display};

use yew::Callback;

use crate::view::{MainMsg, Route};

pub struct MainLink {
    main_callback: Callback<MainMsg>,
    error_message_callback: RefCell<Option<Callback<String>>>,
}

impl MainLink {
    pub fn new(main_callback: Callback<MainMsg>) -> Self {
        Self {
            main_callback,
            error_message_callback: RefCell::new(None),
        }
    }

    pub fn set_error_message_callback(&self, main_callback: Callback<String>) {
        *self.error_message_callback.borrow_mut() = Some(main_callback);
    }

    pub fn go(&self, route: Route) {
        self.main_callback.emit(MainMsg::OnRouteChange(route));
    }

    pub fn emit_error(&self, err: String) {
        if let Some(callback) = self.error_message_callback.borrow().as_ref() {
            callback.emit(err)
        }
    }
}
