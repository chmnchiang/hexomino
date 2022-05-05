use std::{lazy::OnceCell, rc::Rc};

use yew::{html::Scope, Callback, Component};

use self::connection::{Connection, ConnectionError};

pub mod connection;

#[derive(Clone, PartialEq, Eq, Default)]
pub struct MainContext(Rc<OnceCell<MainContextInner>>);

impl MainContext {
    pub fn init_with(&self, connection_error_callback: Callback<ConnectionError>) {
        self.0.set(MainContextInner {
            connection: Rc::new(Connection::new(connection_error_callback)),
        }).map_err(drop).expect("context already initialized");
    }

    pub fn connection(&self) -> Rc<Connection> {
        self.0
            .get()
            .expect("context not initialized")
            .connection
            .clone()
    }
}

#[derive(PartialEq, Eq)]
struct MainContextInner {
    connection: Rc<Connection>,
}

pub trait ScopeExt {
    fn connection(&self) -> Rc<Connection>;
}

impl<T: Component> ScopeExt for Scope<T> {
    fn connection(&self) -> Rc<Connection> {
        let (context, _) = self.context::<MainContext>(Callback::noop()).unwrap();
        context.connection()
    }
}
