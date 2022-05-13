use std::{lazy::OnceCell, rc::Rc};

use yew::{html::Scope, Callback, Component};

use crate::view::MainMsg;

use self::{
    connection::{Connection, ConnectionError},
    main::MainLink,
};

pub mod connection;
pub mod main;

#[derive(Clone, PartialEq, Default)]
pub struct MainContext(Rc<OnceCell<MainContextInner>>);

impl MainContext {
    pub fn init_with(
        &self,
        connection_error_callback: Callback<ConnectionError>,
        main_callback: Callback<MainMsg>,
    ) {
        self.0
            .set(MainContextInner {
                connection: Rc::new(Connection::new(connection_error_callback)),
                main: Rc::new(MainLink::new(main_callback)),
            })
            .map_err(drop)
            .expect("context already initialized");
    }

    pub fn connection(&self) -> Rc<Connection> {
        self.0
            .get()
            .expect("context not initialized")
            .connection
            .clone()
    }

    pub fn main(&self) -> Rc<MainLink> {
        self.0.get().expect("context not initialized").main.clone()
    }
}

struct MainContextInner {
    connection: Rc<Connection>,
    main: Rc<MainLink>,
}

impl PartialEq for MainContextInner {
    fn eq(&self, other: &Self) -> bool {
        self.connection == other.connection
    }
}

pub trait ScopeExt {
    fn connection(&self) -> Rc<Connection>;
    fn main(&self) -> Rc<MainLink>;
}

impl<T: Component> ScopeExt for Scope<T> {
    fn connection(&self) -> Rc<Connection> {
        let (context, _) = self.context::<MainContext>(Callback::noop()).unwrap();
        context.connection()
    }
    fn main(&self) -> Rc<MainLink> {
        let (context, _) = self.context::<MainContext>(Callback::noop()).unwrap();
        context.main()
    }
}
