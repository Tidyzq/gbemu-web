mod bus;
mod cartridge;
mod cpu;
mod emu;
mod instruction;
mod io;
mod ram;
mod utils;

use cartridge::Cartridge;
use emu::Emu;
use std::env;

fn main() -> std::io::Result<()> {
    if let Some(filename) = env::args().nth(1) {
        println!("reading cart {:?}", filename);
        let mut cartridge = Cartridge::load_cartridge(&filename)?;
        println!("loaded cart {:#?}", cartridge);
        Emu::run(&mut cartridge);
        Ok(())
    } else {
        panic!("must pass filename")
    }
}
