extern crate console_error_panic_hook;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::console;

mod lobby;

#[cfg(test)]
mod tests {
    use super::*;
}
