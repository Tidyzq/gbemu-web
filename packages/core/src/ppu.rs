use crate::cpu::BusModule;

#[derive(Clone, Copy)]
pub struct OAMEntry {
    y: u8,
    x: u8,
    tile: u8,
    flags: u8,
}

impl OAMEntry {
    fn new() -> Self {
        OAMEntry {
            y: 0,
            x: 0,
            tile: 0,
            flags: 0,
        }
    }
}

pub trait ScreenWriter {
    fn set_index(&mut self, index: usize, data: u8);
}

pub struct PPU {
    oam_ram: [u8; 0xA0],
    vram: [u8; 0x2000],
    debug_screen_writer: Option<Box<dyn ScreenWriter>>,
    pub dma: DMA,
}

/* R G B A */
static TILE_COLORS: [[u8; 4]; 4] = [
    [0xFF, 0xFF, 0xFF, 0xFF],
    [0xAA, 0xAA, 0xAA, 0xFF],
    [0x55, 0x55, 0x55, 0xFF],
    [0x00, 0x00, 0x00, 0xFF],
];

impl PPU {
    pub fn create() -> Self {
        PPU {
            oam_ram: [0; 0xA0],
            vram: [0; 0x2000],
            debug_screen_writer: None,
            dma: DMA::new(),
        }
    }

    pub fn set_debug_screen_writer(&mut self, writer: Box<dyn ScreenWriter>) {
        self.debug_screen_writer = Some(writer);
        for addr in (0..0x2000).step_by(2) {
            self.write_to_debug_screen(addr);
        }
    }

    pub fn tick() {}

    pub fn oam_read(&self, address: u16) -> u8 {
        let address = address - 0xFE00;
        self.oam_ram[address as usize]
    }

    pub fn oam_write(&mut self, address: u16, value: u8) {
        let address = address - 0xFE00;
        self.oam_ram[address as usize] = value;
    }

    pub fn vram_read(&self, address: u16) -> u8 {
        let address = address - 0x8000;
        self.vram[address as usize]
    }

    pub fn vram_write(&mut self, address: u16, value: u8) {
        let address = address - 0x8000;
        self.vram[address as usize] = value;
        self.write_to_debug_screen(address);
    }

    fn write_to_debug_screen(&mut self, address: u16) {
        let address = address as usize;
        if let Some(debug_screen_writer) = &mut self.debug_screen_writer {
            let line = (address & 0xF) >> 1;
            let tile_index = address >> 4;
            let tile_y = tile_index >> 4;
            let tile_x = tile_index & 0xF;
            let (b1, b2) = if address & 0x1 == 0 {
                (self.vram[address], self.vram[address + 1])
            } else {
                (self.vram[address - 1], self.vram[address])
            };
            for bit in 0..=7 {
                let hi = ((b1 >> (7 - bit)) & 1) << 1;
                let lo = (b2 >> (7 - bit)) & 1;

                let screen_index = ((tile_y * 16 * 8 * 8 + line * 16 * 8 + tile_x * 8) + bit) * 4;
                let color_index = hi | lo;
                let color = TILE_COLORS[color_index as usize];

                for i in 0..=3 {
                    debug_screen_writer.set_index(screen_index + i, color[i]);
                }
            }
        }
    }
}

pub struct DMA {
    pub active: bool,
    byte: u8,
    value: u8,
    start_delay: u8,
}

impl DMA {
    pub fn new() -> Self {
        DMA {
            active: false,
            byte: 0,
            value: 0,
            start_delay: 0,
        }
    }

    pub fn start(&mut self, start: u8) {
        self.active = true;
        self.byte = 0;
        self.start_delay = 2;
        self.value = start;
    }

    pub fn tick(&mut self) -> Option<(u16, u16)> {
        if !self.active {
            return None;
        }

        if self.start_delay != 0 {
            self.start_delay -= 1;
            return None;
        }

        let from = (self.value & 0xFF) as u16 * 0x100 + self.byte as u16;
        let to = self.byte as u16 + 0xFE;

        self.byte += 1;
        self.active = self.byte < 0xA0;

        Some((from, to))
    }
}
