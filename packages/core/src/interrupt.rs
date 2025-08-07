#[derive(Debug)]
pub enum InterruptKind {
    VBlank = 1,
    LCDStat = 2,
    Timer = 4,
    Serial = 8,
    JoyPad = 16,
}

#[derive(Debug)]
pub struct Interrupt {
    pub master_enabled: bool,
    pub enable: u8,
    pub flag: u8,
}

impl Interrupt {
    pub fn create() -> Self {
        Interrupt {
            master_enabled: false,
            enable: 0,
            flag: 0,
        }
    }

    pub fn handle_interrupts(&mut self) -> Option<u16> {
        let flag = self.flag & self.enable;
        if flag == 0 {
            return None;
        }
        let (interrupt, address) = match flag {
            flag if flag & InterruptKind::VBlank as u8 != 0 => (InterruptKind::VBlank, 0x40),
            flag if flag & InterruptKind::LCDStat as u8 != 0 => (InterruptKind::LCDStat, 0x48),
            flag if flag & InterruptKind::Timer as u8 != 0 => (InterruptKind::Timer, 0x50),
            flag if flag & InterruptKind::Serial as u8 != 0 => (InterruptKind::Serial, 0x58),
            flag if flag & InterruptKind::JoyPad as u8 != 0 => (InterruptKind::JoyPad, 0x60),
            _ => unreachable!(),
        };
        self.flag &= !(interrupt as u8);
        self.master_enabled = false;
        Some(address)
    }

    pub fn request_interrupt(&mut self, kind: InterruptKind) {
        self.flag |= kind as u8;
    }
}
