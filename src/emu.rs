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
    pub fn run(cartridge: &mut Cartridge) {
        let mut wram = Ram::<0x2000>::create();
        let mut hram = Ram::<0x80>::create();
        let serial = Rc::new(RefCell::new(Serial::create()));
        let mut io = IO::create(serial);

        let bus = Bus {
            cartridge,
            wram: &mut wram,
            hram: &mut hram,
            io: &mut io,
        };

        let mut cpu = CpuContext::create(bus);

        cpu.init();

        while !cpu.halted {
            if !cpu.step() {
                println!("CPU stopped");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<'a> Emu<'a> {
        pub fn run_test(filename: String, limit: usize) -> std::io::Result<()> {
            let mut cartridge = Cartridge::load_cartridge(&filename)?;
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
            let mut cycles: usize = 0;

            while cycles < limit && !cpu.halted {
                if !cpu.step() {
                    println!("CPU stopped");
                }
                let mut serial = serial.borrow_mut();
                if serial.control == 0x81 {
                    dbg_msg.push(serial.data as char);
                    serial.control = 0;
                    if dbg_msg.ends_with("Passed\n") {
                        return Ok(());
                    }
                }
                cycles += 1
            }
            panic!("failed")
        }
    }

    #[test]
    fn special() -> std::io::Result<()> {
        Emu::run_test("./roms/01-special.gb".into(), 10000000)
    }

    #[test]
    fn interrupts() -> std::io::Result<()> {
        Emu::run_test("./roms/02-interrupts.gb".into(), 10000000)
    }
}
