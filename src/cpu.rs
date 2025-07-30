use crate::bus::Bus;
use crate::instruction::{
    self, get_instruction_by_opcode, AddressingMode, Condition, Instruction, Register,
};

#[derive(Debug, Default)]
struct Registers {
    a: u8,
    f: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    pc: u16,
    sp: u16,
}

fn is_reg_16(reg: Register) -> bool {
    match reg {
        Register::AF | Register::BC | Register::DE | Register::HL | Register::PC | Register::SP => {
            true
        }
        _ => false,
    }
}

#[derive(Debug)]
pub struct CpuContext<'a> {
    registers: Registers,

    pub halted: bool,
    pub stepping: bool,

    int_master_enabled: bool,

    bus: &'a mut Bus<'a>,
}

#[derive(Debug)]
enum DataKind {
    D8(u8),
    D16(u16),
}

impl Into<u16> for DataKind {
    fn into(self) -> u16 {
        match self {
            DataKind::D8(data) => data as u16,
            DataKind::D16(data) => data,
        }
    }
}

impl Into<u8> for DataKind {
    fn into(self) -> u8 {
        match self {
            DataKind::D8(data) => data,
            DataKind::D16(data) => data as u8,
        }
    }
}

impl Into<u16> for &DataKind {
    fn into(self) -> u16 {
        match self {
            DataKind::D8(data) => *data as u16,
            DataKind::D16(data) => *data,
        }
    }
}

impl Into<u8> for &DataKind {
    fn into(self) -> u8 {
        match self {
            DataKind::D8(data) => *data,
            DataKind::D16(data) => *data as u8,
        }
    }
}

#[derive(Debug)]
enum LeftDataKind {
    R(Register),
    MR(Register),
    A16(u16),
}

macro_rules! concat_u16 {
    ($hi:expr,$lo:expr) => {
        ($lo as u16) | (($hi as u16) << 8)
    };
}

impl<'a> CpuContext<'a> {
    pub fn create(bus: &'a mut Bus<'a>) -> Self {
        CpuContext {
            registers: Registers::default(),

            halted: true,
            stepping: false,

            int_master_enabled: false,

            bus,
        }
    }

    pub fn init(&mut self) {
        self.registers.pc = 0x100;
        self.halted = false;
    }

    fn read_bus_16(&self, address: u16) -> u16 {
        let lo = self.bus.read(address) as u16;
        let hi = self.bus.read(address + 1) as u16;

        lo | (hi << 8)
    }

    fn write_bus_16(&mut self, address: u16, value: u16) {
        let lo = (value & 0x00FF) as u8;
        let hi = ((value & 0xFF00) >> 8) as u8;
        self.bus.write(address, lo);
        self.bus.write(address, hi);
    }

    fn read_reg(&self, register: &Register) -> DataKind {
        match register {
            Register::A => DataKind::D8(self.registers.a),
            Register::F => DataKind::D8(self.registers.f),
            Register::B => DataKind::D8(self.registers.b),
            Register::C => DataKind::D8(self.registers.c),
            Register::D => DataKind::D8(self.registers.d),
            Register::E => DataKind::D8(self.registers.e),
            Register::H => DataKind::D8(self.registers.h),
            Register::L => DataKind::D8(self.registers.l),

            Register::AF => DataKind::D16(concat_u16!(self.registers.a, self.registers.f)),
            Register::BC => DataKind::D16(concat_u16!(self.registers.b, self.registers.c)),
            Register::DE => DataKind::D16(concat_u16!(self.registers.d, self.registers.e)),
            Register::HL => DataKind::D16(concat_u16!(self.registers.h, self.registers.l)),

            Register::PC => DataKind::D16(self.registers.pc),
            Register::SP => DataKind::D16(self.registers.sp),
        }
    }

    fn write_reg(&mut self, register_kind: &Register, value: u16) {
        macro_rules! split {
            ($hi:expr,$lo:expr,$val:expr) => {{
                $lo = ($val & 0x00FF) as u8;
                $hi = (($val & 0xFF00) >> 8) as u8;
            }};
        }
        match register_kind {
            Register::A => self.registers.a = value as u8,
            Register::F => self.registers.f = value as u8,
            Register::B => self.registers.b = value as u8,
            Register::C => self.registers.c = value as u8,
            Register::D => self.registers.d = value as u8,
            Register::E => self.registers.e = value as u8,
            Register::H => self.registers.h = value as u8,
            Register::L => self.registers.l = value as u8,

            Register::AF => split!(self.registers.a, self.registers.f, value),
            Register::BC => split!(self.registers.b, self.registers.c, value),
            Register::DE => split!(self.registers.d, self.registers.e, value),
            Register::HL => split!(self.registers.h, self.registers.l, value),

            Register::PC => self.registers.pc = value,
            Register::SP => self.registers.sp = value,
        }
    }

    fn set_flags(&mut self, z: i8, n: i8, h: i8, c: i8) {
        macro_rules! set_flag_bit {
            ($i:expr, $v:expr) => {
                if $v == 1 {
                    self.registers.f |= (1 << $i);
                } else if $v == 0 {
                    self.registers.f &= !(1 << $i);
                }
            };
        }
        set_flag_bit!(7, z);
        set_flag_bit!(6, n);
        set_flag_bit!(5, h);
        set_flag_bit!(4, c);
    }

    fn check_condition(&self, cond: &Condition) -> bool {
        macro_rules! flag_bit {
            ($i:expr) => {
                (self.registers.f & (1 << $i)) != 0
            };
        }
        let flag_z = flag_bit!(7);
        let flag_c = flag_bit!(4);

        match cond {
            Condition::None => true,
            Condition::C => flag_c,
            Condition::NC => !flag_c,
            Condition::Z => flag_z,
            Condition::NZ => !flag_z,
        }
    }

    pub fn stack_push(&mut self, data: u8) {
        self.registers.sp -= 1;
        self.bus.write(self.registers.sp, data);
    }
    pub fn stack_push_16(&mut self, data: u16) {
        self.stack_push(((data >> 8) & 0xFF) as u8);
        self.stack_push((data & 0xFF) as u8);
    }
    pub fn stack_pop(&mut self) -> u8 {
        let data = self.bus.read(self.registers.sp);
        self.registers.sp += 1;
        data
    }
    pub fn stack_pop_16(&mut self) -> u16 {
        let lo = self.stack_pop();
        let hi = self.stack_pop();
        (lo as u16) | ((hi as u16) << 8)
    }

    fn emu_cycles(&self, cycles: u8) {}

    fn fetch_instruction(&mut self) -> &'static Instruction {
        let current_opcode = self.bus.read(self.registers.pc);
        print!(
            "{:04x?}: ({:02x} {:02x} {:02x}) ",
            self.registers.pc,
            current_opcode,
            self.bus.read(self.registers.pc + 1),
            self.bus.read(self.registers.pc + 2)
        );
        self.registers.pc += 1;
        let instruction = get_instruction_by_opcode(current_opcode);
        match instruction {
            Instruction::None => unimplemented!(),
            _ => instruction,
        }
    }

    fn fetch_data(&mut self, addressing_mode: &AddressingMode) -> DataKind {
        match addressing_mode {
            AddressingMode::R(register) => self.read_reg(register),
            AddressingMode::MR(register) => {
                DataKind::D8(self.bus.read(self.read_reg(register).into()))
            }
            AddressingMode::A8 | AddressingMode::D8 => {
                let data = self.bus.read(self.registers.pc);
                self.registers.pc += 1;
                self.emu_cycles(1);
                match addressing_mode {
                    AddressingMode::A8 => DataKind::D8(self.bus.read(0xFF00 | data as u16)),
                    AddressingMode::D8 => DataKind::D8(data),
                    _ => unreachable!(),
                }
            }
            AddressingMode::A16 | AddressingMode::D16 => {
                let lo = self.bus.read(self.registers.pc) as u16;
                self.registers.pc += 1;
                self.emu_cycles(1);

                let hi = self.bus.read(self.registers.pc) as u16;
                self.registers.pc += 1;
                self.emu_cycles(1);

                let data = lo | (hi << 8);
                match addressing_mode {
                    AddressingMode::A16 => DataKind::D8(self.bus.read(data)),
                    AddressingMode::D16 => DataKind::D16(data),
                    _ => unreachable!(),
                }
            }
        }
    }

    fn fetch_left_data(&mut self, addressing_mode: &AddressingMode) -> LeftDataKind {
        match addressing_mode {
            AddressingMode::R(register) => LeftDataKind::R(*register),
            AddressingMode::MR(register) => LeftDataKind::MR(*register),
            AddressingMode::A8 => {
                let data = self.bus.read(self.registers.pc);
                self.registers.pc += 1;
                self.emu_cycles(1);
                LeftDataKind::A16(0xFF00 | data as u16)
            }
            AddressingMode::A16 | AddressingMode::D16 => {
                let data = self.read_bus_16(self.registers.pc);
                self.registers.pc += 2;
                self.emu_cycles(2);
                LeftDataKind::A16(data)
            }
            _ => unreachable!(),
        }
    }

    fn write_data(&mut self, left_data: &LeftDataKind, data: &DataKind) {
        match left_data {
            LeftDataKind::R(register) => self.write_reg(register, data.into()),
            LeftDataKind::MR(register) => {
                let address = self.read_reg(register);
                match data {
                    DataKind::D8(data) => self.bus.write(address.into(), *data),
                    DataKind::D16(data) => self.write_bus_16(address.into(), *data),
                    _ => unreachable!(),
                }
            }
            LeftDataKind::A16(address) => self.bus.write(*address, data.into()),
        }
    }

    fn execute(&mut self, instruction: &Instruction) {
        print!("{:<16} ", format!("{:?}", instruction));

        macro_rules! goto_addr {
            ($cond:expr, $addr:expr, $push:expr) => {{
                let address = $addr;
                if self.check_condition($cond) {
                    if $push {
                        self.emu_cycles(2);
                        self.stack_push_16(self.registers.pc);
                    }
                    self.registers.pc = address;
                    self.emu_cycles(1);
                }
            }};
        }

        match instruction {
            Instruction::NOP => {
                // do nothing
            }
            Instruction::EI => self.int_master_enabled = true,
            Instruction::DI => self.int_master_enabled = false,
            Instruction::JP(condition) => {
                goto_addr!(
                    condition,
                    self.fetch_data(&AddressingMode::D16).into(),
                    false
                );
            }
            Instruction::JR(condition) => {
                let rel: u8 = self.fetch_data(&AddressingMode::D8).into();
                goto_addr!(
                    condition,
                    (self.registers.pc as i16 + rel as i16) as u16,
                    false
                );
            }
            Instruction::PUSH(register) => match self.read_reg(register) {
                DataKind::D16(data) => self.stack_push_16(data),
                DataKind::D8(_) => unreachable!(),
            },
            Instruction::POP(register) => {
                if is_reg_16(*register) {
                    let value = self.stack_pop_16();
                    self.write_reg(register, value);
                } else {
                    unreachable!()
                }
                // TODO set flags
            }
            Instruction::CALL(condition) => {
                goto_addr!(
                    condition,
                    self.fetch_data(&AddressingMode::D16).into(),
                    true
                );
            }
            Instruction::RET(condition) => {
                goto_addr!(condition, self.stack_pop_16(), false);
            }
            Instruction::LD(left_mode, right_mode) => {
                let left = self.fetch_left_data(left_mode);
                let right = self.fetch_data(right_mode);
                self.write_data(&left, &right);
            }
            Instruction::LDD1 => {
                // LD (HL-),A
                let hl = self.read_reg(&Register::HL).into();
                let a = self.registers.a;
                self.bus.write(hl, a);
                self.write_reg(&Register::HL, hl - 1);
            }
            Instruction::LDD2 => {
                // LD A,(HL-)
                let hl = self.read_reg(&Register::HL).into();
                self.registers.a = self.bus.read(hl);
                self.write_reg(&Register::HL, hl - 1);
            }
            // Instruction::DEC(register) => {}
            Instruction::XOR(mode) => {
                let data = match self.fetch_data(mode) {
                    DataKind::D8(data) => data,
                    DataKind::D16(data) => data as u8,
                    _ => unreachable!(),
                };
                self.registers.a = self.registers.a ^ data;
                self.set_flags((self.registers.a == 0) as i8, 0, 0, 0)
            }
            _ => unimplemented!("instruction kind {:?}", instruction),
        };
        println!(
            "AF: {:04X?} BC: {:04X?} DE: {:04X?} HL: {:04X?} SP: {:04X?}",
            concat_u16!(self.registers.a, self.registers.f),
            concat_u16!(self.registers.b, self.registers.c),
            concat_u16!(self.registers.d, self.registers.e),
            concat_u16!(self.registers.h, self.registers.l),
            self.registers.sp,
        );
    }

    pub fn step(&mut self) -> bool {
        if !self.halted {
            let instruction = self.fetch_instruction();
            self.execute(instruction);
        }

        return true;
    }
}
