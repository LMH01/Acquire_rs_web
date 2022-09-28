use web_sys::console;

/// Initialize the main lobby state
#[no_mangle]
pub extern fn init() {
    console_error_panic_hook::set_once();
}
