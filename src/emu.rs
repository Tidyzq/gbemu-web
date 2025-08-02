use std::{cell::RefCell, rc::Rc};

use crate::{
    bus::Bus,
    cartridge::Cartridge,
    cpu::CpuContext,
    io::{Serial, IO},
    ram::Ram,
};

pub struct Emu<'a> {
    cpu: CpuContext<'a>,
}

impl<'a> Emu<'a> {
    pub fn run(filename: &String) -> std::io::Result<()> {
        // println!("reading cart {:?}", filename);
        let mut cartridge = Cartridge::load_cartridge(filename)?;
        // println!("loaded cart {:#?}", cartridge);
        let mut wram = Ram::<0x2000>::create();
        let mut hram = Ram::<0x80>::create();
        let serial = Rc::new(RefCell::new(Serial::create()));
        let mut io = IO::create(Rc::clone(&serial));

        let bus = Bus {
            cartridge: &mut cartridge,
            wram: &mut wram,
            hram: &mut hram,
            io: &mut io,
        };

        let mut cpu = CpuContext::create(bus);

        cpu.init();

        let mut dbg_msg = String::new();

        while !cpu.halted {
            if !cpu.step() {
                println!("CPU stopped");
            }
            let mut serial = serial.borrow_mut();
            if serial.control == 0x81 {
                dbg_msg.push(serial.data as char);
                serial.control = 0;
                println!("DBG: {:?}", dbg_msg);
                if dbg_msg.ends_with("Passed\n") {
                    panic!();
                }
            }
        }

        Ok(())
    }
}
