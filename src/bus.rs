// Memory Map
// 0x0000 - 0x3FFF : 16 KiB ROM bank 00               From cartridge, usually a fixed bank
// 0x4000 - 0x7FFF : 16 KiB ROM Bank 01–NN            From cartridge, switchable bank via mapper (if any)
// 0x8000 - 0x9FFF : 8 KiB Video RAM (VRAM)           In CGB mode, switchable bank 0/1
// 0xA000 - 0xBFFF : 8 KiB External RAM               From cartridge, switchable bank if any
// 0xC000 - 0xCFFF : 4 KiB Work RAM (WRAM)
// 0xD000 - 0xDFFF : 4 KiB Work RAM (WRAM)            In CGB mode, switchable bank 1–7
// 0xE000 - 0xFDFF : Echo RAM (mirror of C000–DDFF)   Nintendo says use of this area is prohibited.
// 0xFE00 - 0xFE9F : Object attribute memory (OAM)
// 0xFEA0 - 0xFEFF : Not Usable                       Nintendo says use of this area is prohibited.
// 0xFF00 - 0xFF7F : I/O Registers
// 0xFF80 - 0xFFFE : High RAM (HRAM)
// 0xFFFF - 0xFFFF : Interrupt Enable register (IE)

// 模拟总线
pub struct Bus<'a> {
    pub cartridge: &'a mut dyn BusModule,
    pub wram: &'a mut dyn BusModule,
    pub hram: &'a mut dyn BusModule,
    pub io: &'a mut dyn BusModule,
}

pub trait BusModule {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, value: u8);
}

impl<'a> Bus<'a> {
    pub fn read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.cartridge.read(address),
            0xC000..=0xDFFF => self.wram.read(address - 0xC000),
            0xFF00..=0xFF7F => self.io.read(address - 0xFF00),
            0xFF80..=0xFFFE => self.hram.read(address - 0xFF80),
            _ => {
                println!("Unsupported bus read at 0x{:X?}", address);
                0
            }
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF | 0xA000..=0xBFFF => self.cartridge.write(address, value),
            0xC000..=0xDFFF => self.wram.write(address - 0xC000, value),
            0xFF00..=0xFF7F => self.io.write(address - 0xFF00, value),
            0xFF80..=0xFFFE => self.hram.write(address - 0xFF80, value),
            _ => println!("Unsupported bus write at 0x{:X?} = {:X?}", address, value),
        }
    }
}
