mod cartridge;
mod cpu;
mod emu;
mod instruction;
mod interrupt;
mod io;
mod ppu;
mod ram;
mod timer;
mod utils;

use cartridge::Cartridge;
use cpu::CpuContext;
use js_sys::{SharedArrayBuffer, Uint8Array};
use ppu::ScreenWriter;
use wasm_bindgen::prelude::*;

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
pub struct Emu {
    cpu: CpuContext,
}

struct SharedArrayBufferWriter {
    buffer: Uint8Array,
}

impl SharedArrayBufferWriter {
    pub fn create(buffer: SharedArrayBuffer) -> Self {
        SharedArrayBufferWriter {
            buffer: Uint8Array::new(&buffer),
        }
    }
}

impl ScreenWriter for SharedArrayBufferWriter {
    fn set_index(&mut self, index: usize, data: u8) {
        self.buffer.set_index(index as u32, data);
    }
}

#[wasm_bindgen]
impl Emu {
    #[wasm_bindgen(constructor)]
    pub fn create(cart_data: &mut [u8], debug_buffer: SharedArrayBuffer) -> Self {
        set_panic_hook();
        let cartridge = Cartridge::from(Vec::from(cart_data));
        let mut cpu = CpuContext::create(cartridge);

        cpu.bus
            .ppu
            .set_debug_screen_writer(Box::new(SharedArrayBufferWriter::create(debug_buffer)));

        Emu { cpu }
    }

    #[wasm_bindgen]
    pub fn run(&mut self) {
        let cpu = &mut self.cpu;

        cpu.init();
        while cpu.step() {}
    }
}
