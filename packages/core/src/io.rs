use std::{cell::RefCell, rc::Rc};

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
    pub serial: Option<Rc<RefCell<Serial>>>,
}

impl IO {
    pub fn create() -> Self {
        IO { serial: None }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xFF01 => match &self.serial {
                Some(serial) => serial.borrow().data,
                None => 0,
            },
            0xFF02 => match &self.serial {
                Some(serial) => serial.borrow().control,
                None => 0,
            },
            _ => {
                // println!("Unsupported IO read at {:X?}", address);
                0
            }
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF01 => match &self.serial {
                Some(serial) => serial.borrow_mut().data = value,
                None => {},
            },
            0xFF02 => match &self.serial {
                Some(serial) => serial.borrow_mut().control = value,
                None => {},
            },
            _ => {} // _ => println!("Unsupported IO write at {:X?} = {:X?}", address, value),
        }
    }
}
