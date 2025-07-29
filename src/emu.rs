use crate::{
    bus::Bus,
    cartridge::Cartridge,
    cpu::CpuContext,
    ram::{HighRam, WorkingRam},
};

pub struct Emu<'a> {
    cpu: CpuContext<'a>,
}

impl<'a> Emu<'a> {
    pub fn run(filename: &String) -> std::io::Result<()> {
        println!("reading cart {:?}", filename);
        let mut cartridge = Cartridge::load_cartridge(filename)?;
        println!("loaded cart {:#?}", cartridge);
        let mut wram = WorkingRam::create();
        let mut hram = HighRam::create();

        let mut bus = Bus {
            cartridge: &mut cartridge,
            wram: &mut wram,
            hram: &mut hram,
        };

        let mut cpu = CpuContext::create(&mut bus);

        cpu.init();

        while !cpu.halted {
            if !cpu.step() {
                println!("CPU stopped")
            }
        }

        Ok(())
    }
}
