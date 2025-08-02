use std::{cell::RefCell, rc::Rc};

use crate::bus::BusModule;

#[derive(Debug)]
struct Timer {
    pub div: u16,
    pub tima: u8,
    pub tma: u8,
    pub tac: u8,
}

impl Timer {
    pub fn create() -> Self {
        Timer {
            div: 0xAC00,
            tima: 0,
            tma: 0,
            tac: 0,
        }
    }
}

#[derive(Debug)]
pub struct Serial {
    pub data: u8,
    pub control: u8,
}

impl Serial {
    pub fn create() -> Self {
        Serial {
            data: 0,
            control: 0,
        }
    }
}

#[derive(Debug)]
pub struct IO {
    pub serial: Rc<RefCell<Serial>>,
    pub timer: Timer,
}

impl IO {
    pub fn create(serial: Rc<RefCell<Serial>>) -> Self {
        IO {
            serial,
            timer: Timer::create(),
        }
    }
}

impl BusModule for IO {
    fn read(&self, address: u16) -> u8 {
        match address {
            0x01 => self.serial.borrow().data,
            0x02 => self.serial.borrow().control,
            0x04 => (self.timer.div >> 8) as u8,
            0x05 => self.timer.tima,
            0x06 => self.timer.tma,
            0x07 => self.timer.tac,
            _ => {
                println!("Unsupported IO read at {:X?}", address);
                0
            }
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x01 => self.serial.borrow_mut().data = value,
            0x02 => self.serial.borrow_mut().control = value,
            0x04 => self.timer.div = 0,
            0x05 => self.timer.tima = value,
            0x06 => self.timer.tma = value,
            0x07 => self.timer.tac = value,
            _ => println!("Unsupported IO write at {:X?} = {:X?}", address, value),
        }
    }
}
