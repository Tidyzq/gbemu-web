use crate::bus::Bus;
use crate::instruction::{AddressingMode, CBInstruction, Condition, Instruction, Register};

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

    fn read_bus(&self, address: u16) -> u8 {
        self.bus.read(address)
    }

    fn write_bus(&mut self, address: u16, value: u8) {
        self.bus.write(address, value);
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
        Instruction::from(current_opcode)
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

        macro_rules! set_flag_bit {
            ($i:expr, $v:literal) => {{
                if $v == 1 {
                    self.registers.f |= (1 << $i);
                } else if $v == 0 {
                    self.registers.f &= !(1 << $i);
                }
            }};
            ($i:expr, $v:expr) => {{
                if $v {
                    self.registers.f |= (1 << $i);
                } else {
                    self.registers.f &= !(1 << $i);
                }
            }};
        }

        macro_rules! set_flags {
            ($z:expr, $n:expr, $h:expr, $c:expr) => {{
                set_flag_bit!(7, $z);
                set_flag_bit!(6, $n);
                set_flag_bit!(5, $h);
                set_flag_bit!(4, $c);
            }};
        }

        macro_rules! get_flag {
            (z) => {
                self.registers.f & (1 << 7) != 0
            };
            (n) => {
                self.registers.f & (1 << 6) != 0
            };
            (h) => {
                self.registers.f & (1 << 5) != 0
            };
            (c) => {
                self.registers.f & (1 << 4) != 0
            };
        }

        match instruction {
            /* Miscellaneous instructions */
            Instruction::NOP => {}
            Instruction::DAA => {
                let mut u: u8 = 0;
                let mut fc = false;

                if get_flag!(h) || (!get_flag!(n) && (self.registers.a & 0xf) > 9) {
                    u = 6;
                }
                if get_flag!(h) || (!get_flag!(n) && self.registers.a > 0x99) {
                    u |= 0x60;
                    fc = true;
                }

                if get_flag!(n) {
                    self.registers.a -= u;
                } else {
                    self.registers.a += u;
                }
                set_flags!(self.registers.a == 0, -1, 0, fc);
            }
            Instruction::STOP => {
                unimplemented!("STOP")
            }
            /* Interrupt-related instructions */
            Instruction::EI => self.int_master_enabled = true,
            Instruction::DI => self.int_master_enabled = false,
            Instruction::HALT => self.halted = true,
            /* Jumps and subroutine instructions */
            Instruction::JP(condition) => {
                goto_addr!(
                    condition,
                    self.fetch_data(&AddressingMode::D16).into(),
                    false
                );
            }
            Instruction::JPHL => {
                goto_addr!(&Condition::None, self.read_reg(&Register::HL).into(), false);
            }
            Instruction::JR(condition) => {
                let rel: u8 = self.fetch_data(&AddressingMode::D8).into();
                let (addr, _) = self.registers.pc.overflowing_add_signed(rel as i8 as i16);
                goto_addr!(condition, addr, false);
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
            /* Stack manipulation instructions */
            Instruction::PUSH(register) => match self.read_reg(register) {
                DataKind::D16(data) => self.stack_push_16(data),
                DataKind::D8(_) => unreachable!(),
            },
            Instruction::POP(register) => {
                if !is_reg_16(*register) {
                    unreachable!()
                }
                let value = self.stack_pop_16();
                self.write_reg(register, value);
            }
            Instruction::POPAF => {
                let value = self.stack_pop_16();
                self.write_reg(&Register::AF, value & 0xFFF0);
            }
            /* Load instructions */
            Instruction::LD(left_mode, right_mode) => {
                let left = self.fetch_left_data(left_mode);
                let right = self.fetch_data(right_mode);
                self.write_data(&left, &right);
            }
            Instruction::LDI1 => {
                // LD (HL+),A
                let hl = self.read_reg(&Register::HL).into();
                let a = self.registers.a;
                self.bus.write(hl, a);
                self.write_reg(&Register::HL, hl + 1);
            }
            Instruction::LDI2 => {
                // LD A,(HL+)
                let hl = self.read_reg(&Register::HL).into();
                self.registers.a = self.bus.read(hl);
                self.write_reg(&Register::HL, hl + 1);
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
            Instruction::LDHL => {
                let rel: u8 = self.fetch_data(&AddressingMode::D8).into();
                let hl: u16 = self.read_reg(&Register::HL).into();
                self.write_reg(&Register::HL, hl.wrapping_add_signed(rel as i8 as i16));
                let h = ((hl & 0xF) + (rel as u16 & 0xF)) >= 0x10;
                let c = ((hl & 0xFF) + (rel as u16 & 0xFF)) >= 0x100;
                set_flags!(0, 0, h, c);
            }
            /* Arithmetic instructions */
            Instruction::INC(register) => match self.read_reg(register) {
                DataKind::D8(data) => {
                    let (new_data, _) = data.overflowing_add(1);
                    self.write_reg(register, new_data as u16);
                    set_flags!(data == 0, 0, (data & 0x0F) == 0, -1);
                }
                DataKind::D16(data) => {
                    let (new_data, _) = data.overflowing_add(1);
                    self.emu_cycles(1);
                    self.write_reg(register, new_data);
                }
            },
            Instruction::DEC(register) => match self.read_reg(register) {
                DataKind::D8(data) => {
                    let (new_data, _) = data.overflowing_sub(1);
                    self.write_reg(register, new_data as u16);
                    set_flags!(data == 0, 1, (data & 0x0F) == 0x0F, -1);
                }
                DataKind::D16(data) => {
                    let (new_data, _) = data.overflowing_sub(1);
                    self.emu_cycles(1);
                    self.write_reg(register, new_data);
                }
            },
            Instruction::INCHL => {
                let register = &Register::HL;
                let addr: u16 = self.read_reg(register).into();
                let (data, _) = self.read_bus(addr).overflowing_add(1);
                self.emu_cycles(1);

                self.write_bus(addr, data);

                set_flags!(data == 0, 0, (data & 0x0F) == 0, -1);
            }
            Instruction::DECHL => {
                let addr: u16 = self.read_reg(&Register::HL).into();
                let (data, _) = self.read_bus(addr).overflowing_sub(1);
                self.emu_cycles(1);

                self.write_bus(addr, data);

                set_flags!(data == 0, 1, (data & 0x0F) == 0x0F, -1);
            }
            Instruction::ADD(mode) => {
                let data: u8 = self.fetch_data(mode).into();
                let (new_data, c) = self.registers.a.overflowing_add(data);
                let z = data == 0;
                let h = (self.registers.a & 0x0F) + (data & 0x0F) >= 0x10;
                self.registers.a = new_data;
                set_flags!(z, 0, h, c);
            }
            Instruction::SUB(mode) => {
                let data: u8 = self.fetch_data(mode).into();
                let (new_data, c) = self.registers.a.overflowing_sub(data);
                let z = data == 0;
                let h = (self.registers.a & 0x0F) < (data & 0x0F);
                self.registers.a = new_data;
                set_flags!(z, 1, h, c);
            }
            Instruction::ADDHL(register) => {
                let data = match self.read_reg(register) {
                    DataKind::D8(_) => unreachable!(),
                    DataKind::D16(data) => data,
                };
                let hl: u16 = self.read_reg(&Register::HL).into();
                self.emu_cycles(1);
                let (new_data, c) = hl.overflowing_add(data);
                let h = (hl & 0x0FFF) + (data & 0x0FFF) >= 0x1000;
                self.write_reg(&Register::HL, new_data);
                set_flags!(-1, 0, h, c);
            }
            Instruction::ADDSP => {
                let data: u8 = self.fetch_data(&AddressingMode::D8).into();
                let sp = self.registers.sp;
                self.emu_cycles(1);
                let (new_data, c) = sp.overflowing_add_signed(data as i8 as i16);
                let h = (sp & 0x0F) + (data as u16 & 0x0F) >= 0x10;
                self.registers.sp = new_data;
                set_flags!(0, 0, h, c);
            }
            Instruction::ADC(mode) => {
                let data: u8 = self.fetch_data(mode).into();
                let a = self.registers.a;
                let old_c: u8 = if get_flag!(c) { 1 } else { 0 };

                let (mut new_data, mut c) = a.overflowing_add(data);
                if get_flag!(c) {
                    let (new_new_data, new_c) = new_data.overflowing_add(1);
                    new_data = new_new_data;
                    c |= new_c;
                }
                self.registers.a = new_data;
                set_flags!(new_data == 0, 0, (a & 0xF) + (data & 0xF) + old_c > 0xF, c);
            }
            /* Bitwise logic instructions */
            Instruction::AND(mode) => {
                let data: u8 = self.fetch_data(mode).into();
                self.registers.a &= data;
                set_flags!(self.registers.a == 0, 0, 0, 0);
            }
            Instruction::CP(mode) => {
                let data: u8 = self.fetch_data(mode).into();
                let a = self.registers.a;
                set_flags!(a == data, 1, a & 0x0F < data & 0x0F, a < data);
            }
            Instruction::OR(mode) => {
                let data: u8 = self.fetch_data(mode).into();
                self.registers.a |= data;
                set_flags!(self.registers.a == 0, 0, 0, 0);
            }
            Instruction::XOR(mode) => {
                let data: u8 = self.fetch_data(mode).into();
                self.registers.a ^= data;
                set_flags!(self.registers.a == 0, 0, 0, 0);
            }
            Instruction::CPL => {
                self.registers.a = !self.registers.a;
                set_flags!(-1, 1, 1, -1);
            }
            /* Bit shift instructions */
            Instruction::RLA => {
                let data: u8 = self.registers.a;
                let c = (data & 0x80) != 0;
                let new_data = (data << 1) | if get_flag!(c) { 1 } else { 0 };
                self.registers.a = new_data;
                set_flags!(0, 0, 0, c);
            }
            Instruction::RLCA => {
                let data: u8 = self.registers.a;
                let c = (data & 0x80) != 0;
                let new_data = data.rotate_left(1);
                self.registers.a = new_data;
                set_flags!(0, 0, 0, c);
            }
            Instruction::RRA => {
                let data: u8 = self.registers.a;
                let c = (data & 0x01) != 0;
                let new_data = (data >> 1) | if get_flag!(c) { 0x80 } else { 0 };
                self.registers.a = new_data;
                set_flags!(0, 0, 0, c);
            }
            Instruction::RRCA => {
                let data: u8 = self.registers.a;
                let c = (data & 0x01) != 0;
                let new_data = data.rotate_right(1);
                self.registers.a = new_data;
                set_flags!(0, 0, 0, c);
            }
            /* Carry flag instructions */
            Instruction::CCF => {
                set_flags!(-1, 0, 0, !get_flag!(c));
            }
            Instruction::SCF => {
                set_flags!(-1, 0, 0, 1);
            }
            /* Prefix CB */
            Instruction::PREFIX => {
                macro_rules! read_reg {
                    ($r:expr) => {{
                        let d: u8 = match $r {
                            Register::HL => self.read_bus(self.read_reg(&Register::HL).into()),
                            reg => self.read_reg(reg).into(),
                        };
                        d
                    }};
                }

                macro_rules! write_reg {
                    ($r:expr, $v:expr) => {{
                        let val = $v;
                        match $r {
                            Register::HL => {
                                self.write_bus(self.read_reg(&Register::HL).into(), val)
                            }
                            reg => self.write_reg(reg, val as u16),
                        };
                    }};
                }

                let opcode: u8 = self.fetch_data(&AddressingMode::D8).into();
                match CBInstruction::from(opcode) {
                    CBInstruction::BIT(bit, reg) => {
                        let data: u8 = read_reg!(reg);
                        set_flags!((data & (1 << bit)) == 0, 0, 1, -1);
                    }
                    CBInstruction::RES(bit, reg) => {
                        let data: u8 = read_reg!(reg);
                        let new_data = data & !(1 << bit);
                        write_reg!(reg, new_data);
                    }
                    CBInstruction::SET(bit, reg) => {
                        let data: u8 = read_reg!(reg);
                        let new_data = data | (1 << bit);
                        write_reg!(reg, new_data);
                    }
                    CBInstruction::RLC(reg) => {
                        let data: u8 = read_reg!(reg);
                        let new_data = data.rotate_left(1);
                        write_reg!(reg, new_data);
                        set_flags!(new_data == 0, 0, 0, new_data & 0x01 != 0);
                    }
                    CBInstruction::RRC(reg) => {
                        let data: u8 = read_reg!(reg);
                        let new_data = data.rotate_right(1);
                        write_reg!(reg, new_data);
                        set_flags!(new_data == 0, 0, 0, data & 0x01 != 0);
                    }
                    CBInstruction::RL(reg) => {
                        let data: u8 = read_reg!(reg);
                        let c = data & (1 << 7) != 0;
                        let new_data = (data << 1) | if get_flag!(c) { 1 } else { 0 };
                        write_reg!(reg, new_data);
                        set_flags!(new_data == 0, 0, 0, c);
                    }
                    CBInstruction::RR(reg) => {
                        let data: u8 = read_reg!(reg);
                        let c = data & 1 != 0;
                        let new_data = (data >> 1) | if get_flag!(c) { 1 << 7 } else { 0 };
                        write_reg!(reg, new_data);
                        set_flags!(new_data == 0, 0, 0, c);
                    }
                    CBInstruction::SLA(reg) => {
                        let data: u8 = read_reg!(reg);
                        let c = data & (1 << 7) != 0;
                        let new_data = data << 1;
                        write_reg!(reg, new_data);
                        set_flags!(new_data == 0, 0, 0, c);
                    }
                    CBInstruction::SRA(reg) => {
                        let data: u8 = read_reg!(reg);
                        let c = data & 1 != 0;
                        let new_data = (data >> 1) | (data & (1 << 7));
                        write_reg!(reg, new_data);
                        set_flags!(new_data == 0, 0, 0, c);
                    }
                    CBInstruction::SWAP(reg) => {
                        let data: u8 = read_reg!(reg);
                        let new_data = ((data & 0xF0) >> 4) | ((data & 0x0F) << 4);
                        write_reg!(reg, new_data);
                        set_flags!(new_data == 0, 0, 0, 0);
                    }
                    CBInstruction::SRL(reg) => {
                        let data: u8 = read_reg!(reg);
                        let c = data & 1 != 0;
                        let new_data = data >> 1;
                        write_reg!(reg, new_data);
                        set_flags!(new_data == 0, 0, 0, c);
                    }

                    inst => unimplemented!("cb instruction kind {:?}", inst),
                }
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
