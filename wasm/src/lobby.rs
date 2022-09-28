use web_sys::console;
use wasm_bindgen::prelude::*;

/// Initialize the main lobby state
#[no_mangle]
pub extern fn init() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub extern fn init_lobby() {
    console_error_panic_hook::set_once();
    console::log_1(&"Hello from Rust!".into());
}

/// Adds the player to the player list
/// `highlighted` -  set true the player will be added highlighted
#[wasm_bindgen]
pub fn add_player(name: &str, highlighted: bool) {
    console::log_1(&name.into());
    let document = web_sys::window().unwrap().document().unwrap();
    let div = document.create_element("li").unwrap();
    if highlighted {
        div.set_class_name("list-group-item list-group-item-primary");
        console::log_1(&"Highlighted".into());
    } else {
        div.set_class_name("list-group-item");
        console::log_1(&"Not Highlighted".into());
    }
    div.set_inner_html(name);
    let _e = document.get_element_by_id("player-list").unwrap().append_child(&div);
}