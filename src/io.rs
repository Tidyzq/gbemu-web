use std::{cell::RefCell, rc::Rc};

use crate::cpu::BusModule;

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
}

impl IO {
    pub fn create(serial: Rc<RefCell<Serial>>) -> Self {
        IO { serial }
    }
}

impl BusModule for IO {
    fn read(&self, address: u16) -> u8 {
        match address {
            0xFF01 => self.serial.borrow().data,
            0xFF02 => self.serial.borrow().control,
            _ => {
                // println!("Unsupported IO read at {:X?}", address);
                0
            }
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF01 => self.serial.borrow_mut().data = value,
            0xFF02 => self.serial.borrow_mut().control = value,
            _ => {} // _ => println!("Unsupported IO write at {:X?} = {:X?}", address, value),
        }
    }
}
