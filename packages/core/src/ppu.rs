use std::ops::RangeInclusive;

use crate::{
    interrupt::InterruptKind,
    utils::{bit, set_bit},
};

static LINES_PER_FRAME: usize = 154;
static OAM_TICKS: usize = 80;
static TICKS_PER_LINE: usize = 456;
static Y_RES: usize = 144;
static X_RES: usize = 160;
static VIDEO_BUFFER_SIZE: usize = Y_RES * X_RES;

#[repr(C)]
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

    #[inline]
    fn bg_priority(&self) -> bool {
        bit!(self.flags, 7)
    }
}

pub trait ScreenWriter {
    fn set_index(&mut self, index: usize, data: u8);
}

pub struct PPU {
    oam_ram: [u8; 0xA0],
    vram: [u8; 0x2000],

    current_frame: usize,
    line_ticks: usize,
    video_buffer: [u32; VIDEO_BUFFER_SIZE],

    debug_screen_writer: Option<Box<dyn ScreenWriter>>,
    pub dma: DMA,
    pub lcd: LCD,
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

            current_frame: 0,
            line_ticks: 0,
            video_buffer: [0; VIDEO_BUFFER_SIZE],

            debug_screen_writer: None,
            dma: DMA::new(),
            lcd: LCD::new(),
        }
    }

    pub fn set_debug_screen_writer(&mut self, writer: Box<dyn ScreenWriter>) {
        self.debug_screen_writer = Some(writer);
        for addr in (0..0x2000).step_by(2) {
            self.write_to_debug_screen(addr);
        }
    }

    pub fn dma_tick<DMAReader>(&mut self, dma_reader: DMAReader)
    where
        DMAReader: FnOnce(&PPU, u16) -> u8,
    {
        if let Some((from, to)) = self.dma.tick() {
            let data = dma_reader(self, from);
            self.oam_ram[to as usize] = data;
        }
    }

    fn increment_ly<RequestInt>(&mut self, request_interrupt: &mut RequestInt)
    where
        RequestInt: FnMut(InterruptKind),
    {
        self.lcd.ly += 1;

        let lyc_equals_ly = self.lcd.ly == self.lcd.ly_compare;
        self.lcd.set_lyc_equals_ly(lyc_equals_ly);

        if lyc_equals_ly && self.lcd.is_lyc_int_selected() {
            request_interrupt(InterruptKind::LCDStat)
        }
    }

    pub fn tick<RequestInt>(&mut self, request_interrupt: &mut RequestInt)
    where
        RequestInt: FnMut(InterruptKind),
    {
        self.line_ticks += 1;

        // println!("line_ticks={:X?} mode={:?}", self.line_ticks, self.lcd.get_ppu_mode());

        match self.lcd.get_ppu_mode() {
            PPUMode::HBlank if self.line_ticks >= TICKS_PER_LINE => {
                self.increment_ly(request_interrupt);
                if self.lcd.ly as usize >= Y_RES {
                    self.lcd.set_ppu_mode(PPUMode::VBlank);
                    request_interrupt(InterruptKind::VBlank);

                    if self.lcd.is_mode1_int_selected() {
                        request_interrupt(InterruptKind::LCDStat)
                    }

                    self.current_frame += 1;
                    self.lcd.ly = 0;

                    // TODO FPS
                } else {
                    self.lcd.set_ppu_mode(PPUMode::OAMScan);
                }
                self.line_ticks = 0;
            }
            PPUMode::VBlank if self.line_ticks >= TICKS_PER_LINE => {
                self.increment_ly(request_interrupt);
                if self.lcd.ly as usize >= LINES_PER_FRAME {
                    self.lcd.set_ppu_mode(PPUMode::OAMScan);
                    self.lcd.ly = 0;
                }
                self.line_ticks = 0;
            }
            PPUMode::OAMScan if self.line_ticks >= OAM_TICKS => {
                self.lcd.set_ppu_mode(PPUMode::Drawing);
            }
            PPUMode::Drawing if self.line_ticks >= OAM_TICKS + 172 /* TODO dynamic ticks */ => {
                self.lcd.set_ppu_mode(PPUMode::HBlank);
            }
            _ => {}
        }
    }

    pub fn oam_read(&self, address: u16) -> u8 {
        if self.dma.active {
            return 0xFF;
        }
        let address = address - 0xFE00;
        self.oam_ram[address as usize]
    }

    pub fn oam_write(&mut self, address: u16, value: u8) {
        if self.dma.active {
            return;
        }
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

    pub fn registers_read(&self, address: u16) -> u8 {
        match address {
            0xFF40 => self.lcd.control,
            0xFF41 => self.lcd.status,
            0xFF42 => self.lcd.scroll_y,
            0xFF43 => self.lcd.scroll_x,
            0xFF44 => self.lcd.ly,
            0xFF45 => self.lcd.ly_compare,
            0xFF46 => self.dma.value,
            0xFF47 => self.lcd.bg_palette,
            0xFF48 => self.lcd.obj_palette[0],
            0xFF49 => self.lcd.obj_palette[1],
            0xFF4A => self.lcd.window_y,
            0xFF4B => self.lcd.window_x,
            _ => unreachable!(),
        }
    }

    pub fn registers_write(&mut self, address: u16, value: u8) {
        println!("PPU registers write ({:X?})={:X?}", address, value);
        match address {
            0xFF40 => self.lcd.control = value,
            0xFF41 => self.lcd.status = value,
            0xFF42 => self.lcd.scroll_y = value,
            0xFF43 => self.lcd.scroll_x = value,
            0xFF44 => {} // LCD ly read only
            0xFF45 => self.lcd.ly_compare = value,
            0xFF46 => self.dma.start(value),
            0xFF47 => {
                self.lcd.bg_palette = value;
                LCD::update_palette(&mut self.lcd.bg_colors, value);
            }
            0xFF48 => {
                self.lcd.obj_palette[0] = value;
                LCD::update_palette(&mut self.lcd.sp1_colors, value & 0xFC);
            }
            0xFF49 => {
                self.lcd.obj_palette[1] = value;
                LCD::update_palette(&mut self.lcd.sp2_colors, value & 0xFC);
            }
            0xFF4A => self.lcd.window_y = value,
            0xFF4B => self.lcd.window_x = value,
            _ => unreachable!(),
        };
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
    pub value: u8,
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
        println!("DMA begin with {:X?}", start);
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
        let to = self.byte as u16;

        println!("DMA copy from {:X?} to {:X?}", from, to);

        self.byte += 1;
        self.active = self.byte < 0xA0;

        if !self.active {
            println!("DMA end");
        }

        Some((from, to))
    }
}

pub struct LCD {
    control: u8,
    status: u8,
    scroll_y: u8,
    scroll_x: u8,
    ly: u8,
    ly_compare: u8,
    bg_palette: u8,
    obj_palette: [u8; 2],
    window_y: u8,
    window_x: u8,

    bg_colors: [[u8; 4]; 4],
    sp1_colors: [[u8; 4]; 4],
    sp2_colors: [[u8; 4]; 4],
}

#[derive(Debug)]
pub enum PPUMode {
    HBlank = 0,
    VBlank = 1,
    OAMScan = 2,
    Drawing = 3,
}

impl LCD {
    pub fn new() -> Self {
        let mut lcd = LCD {
            control: 0x91,
            status: 0,
            scroll_y: 0,
            scroll_x: 0,
            ly: 0,
            ly_compare: 0,
            bg_palette: 0xFC,
            obj_palette: [0xFF, 0xFF],
            window_y: 0,
            window_x: 0,

            bg_colors: TILE_COLORS.clone(),
            sp1_colors: TILE_COLORS.clone(),
            sp2_colors: TILE_COLORS.clone(),
        };
        lcd.set_ppu_mode(PPUMode::OAMScan);
        lcd
    }

    #[inline]
    pub fn is_enabled(&self) -> bool {
        bit!(self.control, 7)
    }

    #[inline]
    pub fn get_window_tile_map_area(&self) -> RangeInclusive<usize> {
        if bit!(self.control, 6) {
            0x9C00..=0x9FFF
        } else {
            0x9800..=0x9BFF
        }
    }

    #[inline]
    pub fn is_window_enabled(&self) -> bool {
        bit!(self.control, 5)
    }

    #[inline]
    pub fn get_bg_window_tile_data_area(&self) -> RangeInclusive<usize> {
        if bit!(self.control, 4) {
            0x8000..=0x8FFF
        } else {
            0x8800..=0x97FF
        }
    }

    #[inline]
    pub fn get_bg_tile_map_area(&self) -> RangeInclusive<usize> {
        if bit!(self.control, 3) {
            0x9C00..=0x9FFF
        } else {
            0x9800..=0x9BFF
        }
    }

    #[inline]
    pub fn get_obj_size(&self) -> (u8, u8) {
        if bit!(self.control, 2) {
            (8, 8)
        } else {
            (8, 16)
        }
    }

    #[inline]
    pub fn is_obj_enable(&self) -> bool {
        bit!(self.control, 1)
    }

    #[inline]
    pub fn is_bg_window_enabled(&self) -> bool {
        bit!(self.control, 0)
    }

    #[inline]
    pub fn get_ppu_mode(&self) -> PPUMode {
        match self.status & 0b11 {
            x if x == PPUMode::HBlank as u8 => PPUMode::HBlank,
            x if x == PPUMode::VBlank as u8 => PPUMode::VBlank,
            x if x == PPUMode::OAMScan as u8 => PPUMode::OAMScan,
            x if x == PPUMode::Drawing as u8 => PPUMode::Drawing,
            _ => unreachable!(),
        }
    }

    #[inline]
    pub fn set_ppu_mode(&mut self, mode: PPUMode) {
        self.status = (self.status & !0b11) | mode as u8
    }

    #[inline]
    pub fn is_lyc_int_selected(&self) -> bool {
        bit!(self.status, 6)
    }

    #[inline]
    pub fn is_mode2_int_selected(&self) -> bool {
        bit!(self.status, 5)
    }

    #[inline]
    pub fn is_mode1_int_selected(&self) -> bool {
        bit!(self.status, 4)
    }

    #[inline]
    pub fn is_mode0_int_selected(&self) -> bool {
        bit!(self.status, 3)
    }

    #[inline]
    pub fn is_lyc_equals_ly(&self) -> bool {
        bit!(self.status, 2)
    }

    #[inline]
    pub fn set_lyc_equals_ly(&mut self, equals: bool) {
        set_bit!(self.status, 2, equals)
    }

    fn update_palette(color: &mut [[u8; 4]; 4], data: u8) {
        for i in 0..4 {
            color[i] = TILE_COLORS[(data as usize >> (i * 2)) & 0b11];
        }
    }
}
