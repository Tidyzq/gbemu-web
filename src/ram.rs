use crate::bus::BusModule;

#[derive(Debug)]
pub struct Ram<const SIZE: usize> {
    pub data: [u8; SIZE],
}

impl<const SIZE: usize> Ram<SIZE> {
    pub fn create() -> Self {
        Ram { data: [0; SIZE] }
    }
}

impl<const SIZE: usize> BusModule for Ram<SIZE> {
    fn read(&self, address: u16) -> u8 {
        self.data[address as usize]
    }

    fn write(&mut self, address: u16, value: u8) {
        self.data[address as usize] = value
    }
}
