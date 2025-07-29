#[derive(Debug)]
pub struct WorkingRam {
    pub data: [u8; 0x2000],
}

impl WorkingRam {
    pub fn create() -> Self {
        WorkingRam { data: [0; 0x2000] }
    }

    pub fn read(&self, address: u16) -> u8 {
        self.data[address as usize]
    }

    pub fn write(&mut self, address: u16, value: u8) {
        self.data[address as usize] = value
    }
}

#[derive(Debug)]
pub struct HighRam {
    pub data: [u8; 0x80],
}

impl HighRam {
    pub fn create() -> Self {
        HighRam { data: [0; 0x80] }
    }

    pub fn read(&self, address: u16) -> u8 {
        self.data[address as usize]
    }

    pub fn write(&mut self, address: u16, value: u8) {
        self.data[address as usize] = value
    }
}
