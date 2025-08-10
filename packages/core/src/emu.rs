use std::{cell::RefCell, rc::Rc};

use crate::{
    cartridge::Cartridge,
    cpu::CpuContext,
    io::{Serial, IO},
    ppu::PPU,
    ram::RAM,
};

pub struct Emu {
    cpu: CpuContext,
}

impl Emu {
    // pub fn run(
    //     cartridge: &mut Cartridge,
    //     screen_buffer: &mut [u8],
    //     debug_screen_buffer: &mut [u8],
    // ) {
    //     let mut wram = Ram::<0x2000, 0xC000>::create();
    //     let mut hram = Ram::<0x80, 0xFF80>::create();
    //     let serial = Rc::new(RefCell::new(Serial::create()));
    //     let mut ppu = PPU::create(screen_buffer, debug_screen_buffer);
    //     let mut io = IO::create(Rc::clone(&serial));

    //     let mut cpu = CpuContext::create(cartridge, &mut wram, &mut hram, &mut ppu, &mut io);

    //     cpu.init();

    //     while cpu.step() {}
    // }
}
