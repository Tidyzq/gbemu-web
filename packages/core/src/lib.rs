mod cartridge;
mod cpu;
mod emu;
mod instruction;
mod interrupt;
mod io;
mod ram;
mod timer;
mod utils;

use cartridge::Cartridge;
use emu::Emu;
use wasm_bindgen::prelude::wasm_bindgen;

fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn emu_run(cart_data: &mut [u8]) {
    set_panic_hook();
    let vec = Vec::from(cart_data);
    let mut cartridge = Cartridge::from(vec);
    Emu::run(&mut cartridge);
}
