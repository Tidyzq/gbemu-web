use crate::utils::{array, array_map};
use enum_map::{enum_map, Enum, EnumMap};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AddressingMode {
    R(Register),
    MR(Register),
    D16,
    D8,
    A16,
    A8,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Register {
    A,
    F,
    B,
    C,
    D,
    E,
    H,
    L,
    AF,
    BC,
    DE,
    HL,
    SP,
    PC,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Instruction {
    None,
    ADC(Register, AddressingMode),
    ADD(Register, AddressingMode),
    ADDSP,
    AND(AddressingMode),
    BIT,
    CALL(Condition),
    CCF,
    CP(AddressingMode),
    CPL,
    DAA,
    DEC(Register),
    DECHL,
    DI,
    EI,
    HALT,
    ILLEGAL_D3,
    ILLEGAL_DB,
    ILLEGAL_DD,
    ILLEGAL_E3,
    ILLEGAL_E4,
    ILLEGAL_EB,
    ILLEGAL_EC,
    ILLEGAL_ED,
    ILLEGAL_F4,
    ILLEGAL_FC,
    ILLEGAL_FD,
    INC(Register),
    INCHL,
    JP(Condition),
    JPHL,
    JR(Condition),
    LD(AddressingMode, AddressingMode),
    LDI1,
    LDI2,
    LDD1,
    LDD2,
    LDHL,
    NOP,
    OR(AddressingMode),
    POP(Register),
    PREFIX,
    PUSH(Register),
    RES,
    RET(Condition),
    RETI,
    RL,
    RLA,
    RLC,
    RLCA,
    RR,
    RRA,
    RRC,
    RRCA,
    RST(u8),
    SBC(Register, AddressingMode),
    SCF,
    SET,
    SLA,
    SRA,
    SRL,
    STOP,
    SUB(AddressingMode),
    SWAP,
    XOR(AddressingMode),
    XORHL,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Condition {
    None,
    NZ,
    Z,
    NC,
    C,
}

macro_rules! AM {
    (d8) => {
        AddressingMode::D8
    };
    (d16) => {
        AddressingMode::D16
    };
    (a8) => {
        AddressingMode::A8
    };
    (a16) => {
        AddressingMode::A16
    };
    (a16) => {
        AddressingMode::A16
    };
    (($t:tt)) => {
        AddressingMode::MR(R!($t))
    };
    ($t:tt) => {
        AddressingMode::R(R!($t))
    };
}

macro_rules! R {
    ($t:tt) => {
        Register::$t
    };
}

macro_rules! C {
    ($t:tt) => {
        Condition::$t
    };
}

macro_rules! INST {
    ([LD $l:tt,$r:tt]) => {
        Instruction::LD(AM!($l), AM!($r))
    };
    ([INC $l:tt]) => {
        Instruction::INC(R!($l))
    };
    ([DEC $l:tt]) => {
        Instruction::DEC(R!($l))
    };
    ([ADC $l:tt,$r:tt]) => {
        Instruction::ADC(R!($l), AM!($r))
    };
    ([ADD $l:tt,$r:tt]) => {
        Instruction::ADD(R!($l), AM!($r))
    };
    ([JR $c:tt]) => {
        Instruction::JR(C!($c))
    };
    ([SBC $l:tt,$r:tt]) => {
        Instruction::SBC(R!($l), AM!($r))
    };
    ([SUB $l:tt]) => {
        Instruction::SUB(AM!($l))
    };
    ([XOR $l:tt]) => {
        Instruction::XOR(AM!($l))
    };
    ([AND $l:tt]) => {
        Instruction::AND(AM!($l))
    };
    ([OR $l:tt]) => {
        Instruction::OR(AM!($l))
    };
    ([CP $l:tt]) => {
        Instruction::CP(AM!($l))
    };
    ([RET $l:tt]) => {
        Instruction::RET(C!($l))
    };
    ([POP $l:tt]) => {
        Instruction::POP(R!($l))
    };
    ([PUSH $l:tt]) => {
        Instruction::PUSH(R!($l))
    };
    ([JP $l:tt]) => {
        Instruction::JP(C!($l))
    };
    ([CALL $l:tt]) => {
        Instruction::CALL(C!($l))
    };
    ([RST $l:tt]) => {
        Instruction::RST($l)
    };
    ($t:tt) => {
        Instruction::$t
    };
}

macro_rules! INST_MAP {
    ($($val:tt);* $(;)?) => {{
        let mut a = [Instruction::None; 0x100];
        let mut idx = 0;
        $(
            a[idx] = INST!($val);
            idx += 1;
        )*
        a
    }};
    // ($ins:expr($l:expr), $($t:expr),*) => {
    //     INST!($ins)
    // };
    // ($ins:expr($l:expr, $r:expr), $($t:expr),*) => {
    //     INST!($ins)
    // };
}

static INSTRUCTIONS: [Instruction; 0x100] = INST_MAP![
/*          x0              x1              x2              x3              x4              x5              x6              x7              x8              x9              xA              xB              xC              xD              xE              xF  */
/* 0x */    NOP;            [LD BC, d16];   [LD (BC), A];   [INC BC];       [INC B];        [DEC B];        [LD B, d8];     RLCA;           [LD a16, SP];   [ADD HL, BC];   [LD A, (BC)];   [DEC BC];       [INC C];        [DEC C];        [LD C, d8];     RRCA;
/* 1x */    STOP;           [LD DE, d16];   [LD (DE), A];   [INC DE];       [INC D];        [DEC D];        [LD D, d8];     RLA;            [JR None];      [ADD HL, DE];   [LD A, (DE)];   [DEC DE];       [INC E];        [DEC E];        [LD E, d8];     RRA;
/* 2x */    [JR NZ];        [LD HL, d16];   LDI1;           [INC HL];       [INC H];        [DEC H];        [LD H, d8];     DAA;            [JR Z];         [ADD HL, HL];   LDI2;           [DEC HL];       [INC L];        [DEC L];        [LD L, d8];     CPL;
/* 3x */    [JR NC];        [LD SP, d16];   LDD1;           [INC SP];       INCHL;          DECHL;          [LD (HL), d8];  SCF;            [JR C];         [ADD HL, SP];   LDD2;           [DEC SP];       [INC A];        [DEC A];        [LD A, d8];     CCF;
/* 4x */    [LD B, B];      [LD B, C];      [LD B, D];      [LD B, E];      [LD B, H];      [LD B, L];      [LD B, (HL)];   [LD B, A];      [LD C, B];      [LD C, C];      [LD C, D];      [LD C, E];      [LD C, H];      [LD C, L];      [LD C, (HL)];   [LD C, A];
/* 5x */    [LD D, B];      [LD D, C];      [LD D, D];      [LD D, E];      [LD D, H];      [LD D, L];      [LD D, (HL)];   [LD D, A];      [LD E, B];      [LD E, C];      [LD E, D];      [LD E, E];      [LD E, H];      [LD E, L];      [LD E, (HL)];   [LD E, A];
/* 6x */    [LD H, B];      [LD H, C];      [LD H, D];      [LD H, E];      [LD H, H];      [LD H, L];      [LD H, (HL)];   [LD H, A];      [LD L, B];      [LD L, C];      [LD L, D];      [LD L, E];      [LD L, H];      [LD L, L];      [LD L, (HL)];   [LD L, A];
/* 7x */    [LD (HL), B];   [LD (HL), C];   [LD (HL), D];   [LD (HL), E];   [LD (HL), H];   [LD (HL), L];   HALT;           [LD (HL), A];   [LD A, B];      [LD A, C];      [LD A, D];      [LD A, E];      [LD A, H];      [LD A, L];      [LD A, (HL)];   [LD A, A];
/* 8x */    [ADD A, B];     [ADD A, C];     [ADD A, D];     [ADD A, E];     [ADD A, H];     [ADD A, L];     [ADD A, (HL)];  [ADD A, A];     [ADC A, B];     [ADC A, C];     [ADC A, D];     [ADC A, E];     [ADC A, H];     [ADC A, L];     [ADC A, (HL)];  [ADC A, A];
/* 9x */    [SUB B];        [SUB C];        [SUB D];        [SUB E];        [SUB H];        [SUB L];        [SUB (HL)];     [SUB A];        [SBC A, B];     [SBC A, C];     [SBC A, D];     [SBC A, E];     [SBC A, H];     [SBC A, L];     [SBC A, (HL)];  [SBC A, A];
/* Ax */    [AND B];        [AND C];        [AND D];        [AND E];        [AND H];        [AND L];        [AND (HL)];     [AND A];        [XOR B];        [XOR C];        [XOR D];        [XOR E];        [XOR H];        [XOR L];        [XOR (HL)];     [XOR A];
/* Bx */    [OR B];         [OR C];         [OR D];         [OR E];         [OR H];         [OR L];         [OR (HL)];      [OR A];         [CP B];         [CP C];         [CP D];         [CP E];         [CP H];         [CP L];         [CP (HL)];      [CP A];
/* Cx */    [RET NZ];       [POP BC];       [JP NZ];        [JP None];      [CALL NZ];      [PUSH BC];      [ADD A, d8];    [RST 0x00];     [RET Z];        [RET None];     [JP Z];         PREFIX;         [CALL Z];       [CALL None];    [ADC A, d8];    [RST 0x08];
/* Dx */    [RET NC];       [POP DE];       [JP NC];        None;           [CALL NC];      [PUSH DE];      [SUB d8];       [RST 0x10];     [RET C];        RETI;           [JP C];         None;           [CALL C];       None;           [SBC A, d8];    [RST 0x18];
/* Ex */    [LD a8, A];     [POP HL];       [LD (C), A];    None;           None;           [PUSH HL];      [AND d8];       [RST 0x20];     ADDSP;          JPHL;           [LD a16, A];    None;           None;           None;           [XOR d8];       [RST 0x28];
/* Fx */    [LD A, a8];     [POP AF];       [LD A, (C)];    DI;             None;           [PUSH AF];      [OR d8];        [RST 0x30];     LDHL;           [LD SP, HL];    [LD A, a16];    EI;             None;           None;           [CP d8];        [RST 0x38];
];

pub fn get_instruction_by_opcode(opcode: u8) -> &'static Instruction {
    &INSTRUCTIONS[opcode as usize]
}
