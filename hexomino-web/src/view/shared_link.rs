use std::{
    cell::RefCell,
    fmt::Debug,
    rc::{Rc, Weak},
};

use yew::{html::Scope, Component};

pub struct SharedLink<COMP: Component>(Rc<RefCell<Option<Scope<COMP>>>>);

impl<COMP: Component> Clone for SharedLink<COMP> {
    fn clone(&self) -> Self {
        SharedLink(Rc::clone(&self.0))
    }
}

impl<COMP: Component> Debug for SharedLink<COMP> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SharedLink({:?}, {})",
            Rc::as_ptr(&self.0),
            if self.0.borrow().is_some() {
                "installed"
            } else {
                "empty"
            }
        )
    }
}

pub struct WeakLink<COMP: Component>(Weak<RefCell<Option<Scope<COMP>>>>);

impl<COMP: Component> SharedLink<COMP> {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(None)))
    }

    pub fn install(&self, link: Scope<COMP>) {
        let mut inner = self.0.borrow_mut();
        *inner = Some(link);
    }

    pub fn downgrade(&self) -> WeakLink<COMP> {
        let weak = Rc::downgrade(&self.0);
        WeakLink(weak)
    }

    pub fn get(&self) -> Scope<COMP> {
        let inner = self.0.borrow();
        inner.as_ref().unwrap().clone()
    }
}

impl<COMP: Component> Default for SharedLink<COMP> {
    fn default() -> Self {
        Self::new()
    }
}

impl<COMP: Component> WeakLink<COMP> {
    pub fn upgrade(&self) -> Option<SharedLink<COMP>> {
        let inner = self.0.upgrade()?;
        Some(SharedLink(inner))
    }
}

impl<COMP: Component> Default for WeakLink<COMP> {
    fn default() -> Self {
        WeakLink(Weak::new())
    }
}

impl<COMP: Component> PartialEq for SharedLink<COMP> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
