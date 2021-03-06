#![feature(if_let_guard)]
#![feature(is_some_with)]
#![feature(let_chains)]
#![feature(let_else)]
#![feature(once_cell)]
#![feature(try_blocks)]
#![allow(dead_code)]

mod context;
mod game;
mod util;
mod view;

use view::App;
use wasm_bindgen::prelude::*;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub fn main() -> Result<(), JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    wasm_logger::init(wasm_logger::Config::default());
    log::info!("start main");
    yew::start_app::<App>();
    log::info!("registered yew component");
    Ok(())
}
