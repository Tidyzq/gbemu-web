#[derive(Debug)]
pub struct RAM<const SIZE: usize, const OFFSET: usize> {
    pub data: [u8; SIZE],
}

impl<const SIZE: usize, const OFFSET: usize> RAM<SIZE, OFFSET> {
    pub fn create() -> Self {
        RAM { data: [0; SIZE] }
    }

    pub fn read(&self, address: u16) -> u8 {
        self.data[address as usize - OFFSET]
    }

    pub fn write(&mut self, address: u16, value: u8) {
        self.data[address as usize - OFFSET] = value
    }
}
