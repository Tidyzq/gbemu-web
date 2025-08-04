use std::{cell::RefCell, rc::Rc};

use crate::{
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
        let mut io = IO::create(Rc::clone(&serial));

        let mut cpu = CpuContext::create(cartridge, &mut wram, &mut hram, &mut io);

        cpu.init();

        while cpu.step() {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl<'a> Emu<'a> {
        pub fn run_test(filename: String, limit: usize) -> std::io::Result<String> {
            let mut cartridge = Cartridge::load_cartridge(&filename)?;
            let mut wram = Ram::<0x2000>::create();
            let mut hram = Ram::<0x80>::create();
            let serial = Rc::new(RefCell::new(Serial::create()));
            let mut io = IO::create(Rc::clone(&serial));

            let mut cpu = CpuContext::create(&mut cartridge, &mut wram, &mut hram, &mut io);

            cpu.init();

            let mut dbg_msg = String::new();
            let mut cycles: usize = 0;

            while cycles < limit {
                if !cpu.step() {
                    println!("CPU stopped");
                }
                let mut serial = serial.borrow_mut();
                if serial.control == 0x81 {
                    dbg_msg.push(serial.data as char);
                    serial.control = 0;
                }
                cycles += 1
            }
            Ok(dbg_msg)
        }
    }

    #[test]
    fn special() -> std::io::Result<()> {
        assert_eq!(
            Emu::run_test("./roms/01-special.gb".into(), 10000000)?,
            "01-special\n\n\nPassed\n",
        );
        Ok(())
    }

    #[test]
    fn interrupts() -> std::io::Result<()> {
        assert_eq!(
            Emu::run_test("./roms/02-interrupts.gb".into(), 10000000)?,
            "02-interrupts\n\n\nPassed\n",
        );
        Ok(())
    }
}
