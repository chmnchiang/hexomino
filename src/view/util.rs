use std::{rc::Rc, cell::RefCell};

type Shared<T> = Rc<RefCell<T>>;
