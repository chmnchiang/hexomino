#![allow(dead_code)]

mod view;

use log::*;

use view::MainView;
use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub fn main() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    wasm_logger::init(wasm_logger::Config::default());
    info!("start main");
    yew::start_app::<MainView>();
    info!("registered yew component");
    Ok(())
}
