use std::{
    cell::UnsafeCell,
    rc::{Rc, Weak},
};

use yew::{html::Scope, Component};

#[derive(Clone)]
pub struct SharedLink<COMP: Component>(Rc<UnsafeCell<Option<Scope<COMP>>>>);

pub struct WeakLink<COMP: Component>(Weak<UnsafeCell<Option<Scope<COMP>>>>);

impl<COMP: Component> SharedLink<COMP> {
    pub fn new() -> Self {
        Self(Rc::new(UnsafeCell::new(None)))
    }

    pub fn install(&self, link: Scope<COMP>) {
        let inner = unsafe { &mut *self.0.get() };
        assert!(inner.is_none());
        *inner = Some(link);
    }

    pub fn downgrade(&self) -> WeakLink<COMP> {
        let weak = Rc::downgrade(&self.0);
        WeakLink(weak)
    }

    pub fn get(&self) -> &Scope<COMP> {
        let inner = unsafe { &*self.0.get() };
        inner.as_ref().unwrap()
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
