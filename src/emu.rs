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
    fn rom_test01() -> std::io::Result<()> {
        assert_eq!(
            Emu::run_test("./roms/01-special.gb".into(), 1500000)?,
            "01-special\n\n\nPassed\n",
        );
        Ok(())
    }

    #[test]
    fn rom_test02() -> std::io::Result<()> {
        assert_eq!(
            Emu::run_test("./roms/02-interrupts.gb".into(), 1000000)?,
            "02-interrupts\n\n\nPassed\n",
        );
        Ok(())
    }

    #[test]
    fn rom_test03() -> std::io::Result<()> {
        assert_eq!(
            Emu::run_test("./roms/03-op sp,hl.gb".into(), 1500000)?,
            "03-op sp,hl\n\n\nPassed\n",
        );
        Ok(())
    }

    #[test]
    fn rom_test04() -> std::io::Result<()> {
        assert_eq!(
            Emu::run_test("./roms/04-op r,imm.gb".into(), 1500000)?,
            "04-op r,imm\n\n\nPassed\n",
        );
        Ok(())
    }

    #[test]
    fn rom_test05() -> std::io::Result<()> {
        assert_eq!(
            Emu::run_test("./roms/05-op rp.gb".into(), 2000000)?,
            "05-op rp\n\n\nPassed\n",
        );
        Ok(())
    }

    #[test]
    fn rom_test06() -> std::io::Result<()> {
        assert_eq!(
            Emu::run_test("./roms/06-ld r,r.gb".into(), 500000)?,
            "06-ld r,r\n\n\nPassed\n",
        );
        Ok(())
    }

    #[test]
    fn rom_test07() -> std::io::Result<()> {
        assert_eq!(
            Emu::run_test("./roms/07-jr,jp,call,ret,rst.gb".into(), 500000)?,
            "07-jr,jp,call,ret,rst\n\n\nPassed\n",
        );
        Ok(())
    }

    #[test]
    fn rom_test08() -> std::io::Result<()> {
        assert_eq!(
            Emu::run_test("./roms/08-misc instrs.gb".into(), 500000)?,
            "08-misc instrs\n\n\nPassed\n",
        );
        Ok(())
    }

    #[test]
    fn rom_test09() -> std::io::Result<()> {
        assert_eq!(
            Emu::run_test("./roms/09-op r,r.gb".into(), 5000000)?,
            "09-op r,r\n\n\nPassed\n",
        );
        Ok(())
    }

    #[test]
    fn rom_test10() -> std::io::Result<()> {
        assert_eq!(
            Emu::run_test("./roms/10-bit ops.gb".into(), 8000000)?,
            "10-bit ops\n\n\nPassed\n",
        );
        Ok(())
    }

    #[test]
    fn rom_test11() -> std::io::Result<()> {
        assert_eq!(
            Emu::run_test("./roms/11-op a,(hl).gb".into(), 8000000)?,
            "11-op a,(hl)\n\n\nPassed\n",
        );
        Ok(())
    }

    // #[test]
    // fn cpu_instrs() -> std::io::Result<()> {
    //     assert_eq!(
    //         Emu::run_test("./roms/cpu_instrs.gb".into(), 10000000)?,
    //         "cpu_instrs\n\n\nPassed\n",
    //     );
    //     Ok(())
    // }
}
