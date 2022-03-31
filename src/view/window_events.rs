use anyhow::{Result, bail};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{KeyboardEvent, window, FocusEvent};
use yew::Callback;

pub struct KeyDownListener {
    closure: Closure<dyn Fn(KeyboardEvent)>,
}

impl KeyDownListener {
    pub fn register(callback: Callback<KeyboardEvent>) -> Result<Self> {
        let closure = Closure::wrap(Box::new(move |e| {
            callback.emit(e);
        }) as Box<dyn Fn(KeyboardEvent)>);
        if let Err(_) = window()
            .unwrap()
            .document()
            .unwrap()
            .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
        {
            bail!("Can't add onkeydown event listener");
        }
        Ok(Self { closure })
    }
}

impl Drop for KeyDownListener {
    fn drop(&mut self) {
        let _ = window()
            .unwrap()
            .document()
            .unwrap()
            .remove_event_listener_with_callback(
                "onkeydown",
                self.closure.as_ref().unchecked_ref(),
            );
    }
}

pub struct WindowResizeListener {
    closure: Closure<dyn Fn(FocusEvent)>,
}

impl WindowResizeListener {
    pub fn register(callback: Callback<FocusEvent>) -> Result<Self> {
        let closure = Closure::wrap(Box::new(move |e| {
            callback.emit(e);
        }) as Box<dyn Fn(FocusEvent)>);
        if let Err(_) = window()
            .unwrap()
            .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
        {
            bail!("Can't add onresize event listener");
        }
        Ok(Self { closure })
    }
}

impl Drop for WindowResizeListener {
    fn drop(&mut self) {
        let _ = window()
            .unwrap()
            .remove_event_listener_with_callback(
                "resize",
                self.closure.as_ref().unchecked_ref(),
            );
    }
}
