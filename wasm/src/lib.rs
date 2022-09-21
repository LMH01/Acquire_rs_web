extern crate console_error_panic_hook;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console;

#[no_mangle]
pub extern fn init() {
    console_error_panic_hook::set_once();
    console::log_1(&"Hello from Rust!".into());
}

#[cfg(test)]
mod tests {
    use super::*;
}
