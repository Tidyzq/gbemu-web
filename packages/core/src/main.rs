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
use std::{env, io::Read};

pub fn load_cartridge(filename: &String) -> std::io::Result<Cartridge> {
    let mut file = std::fs::File::open(filename)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(Cartridge { data })
}

fn main() -> std::io::Result<()> {
    if let Some(filename) = env::args().nth(1) {
        // println!("reading cart {:?}", filename);
        let mut cartridge = load_cartridge(&filename)?;
        // println!("loaded cart {:#?}", cartridge);
        Emu::run(&mut cartridge);
        Ok(())
    } else {
        panic!("must pass filename")
    }
}
