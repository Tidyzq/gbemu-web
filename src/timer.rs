use crate::cpu::BusModule;

#[derive(Debug)]
pub struct Timer {
    pub tick: u64,
    pub div: u16,
    pub tima: u8,
    pub tma: u8,
    pub tac: u8,
}

impl Timer {
    pub fn create() -> Self {
        let mut timer = Timer {
            tick: 0,
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
        };
        timer.init();
        timer
    }

    pub fn init(&mut self) {
        self.div = 0xABCC;
        self.tima = 0;
        self.tma = 0;
        self.tac = 0;
    }

    pub fn tick(&mut self) -> bool {
        self.tick = self.tick.overflowing_add(1).0;
        self.div = self.div.overflowing_add(1).0;

        let update = match self.tac & (0b111) {
            0b101 => self.div & (0xF) == 0,
            0b110 => self.div & (0x3F) == 0,
            0b111 => self.div & (0xFF) == 0,
            0b100 => self.div & (0x3FF) == 0,
            _ => false,
        };
        if update {
            self.tima = self.tima.overflowing_add(1).0;
            if self.tima == 0xFF {
                self.tima = self.tma;
                return true;
            }
        }
        false
    }
}

impl BusModule for Timer {
    fn read(&self, address: u16) -> u8 {
        match address {
            0xFF04 => (self.div >> 8) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac,
            _ => unreachable!(),
        }
    }
    fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF04 => self.div = 0,
            0xFF05 => self.tima = value,
            0xFF06 => self.tma = value,
            0xFF07 => self.tac = value,
            _ => unreachable!(),
        }
    }
}
