mod bus;
mod cartridge;
mod cpu;
mod emu;
mod instruction;
mod ram;
mod utils;

use emu::Emu;

fn main() -> std::io::Result<()> {
    // let mut buffer = "/Users/chyizheng/workspace/gbemu-rs/roms/Wario_Land_3.gbc".into();
    // let buffer = "/Users/chyizheng/workspace/gbemu-rs/roms/tetris.gb".into();
    let buffer = "/Users/chyizheng/workspace/gbemu-rs/roms/cpu_instrs.gb".into();
    Emu::run(&buffer)?;
    Ok(())
}
