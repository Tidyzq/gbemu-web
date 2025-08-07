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
    ADC(AddressingMode),
    ADD(AddressingMode),
    ADDHL(Register),
    ADDSP,
    AND(AddressingMode),
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
    POPAF,
    PREFIX,
    PUSH(Register),
    RET(Condition),
    RETI,
    RLA,
    RLCA,
    RRA,
    RRCA,
    RST(u8),
    SBC(AddressingMode),
    SCF,
    STOP,
    SUB(AddressingMode),
    XOR(AddressingMode),
}

impl Instruction {
    pub fn from(opcode: u8) -> &'static Self {
        &INSTRUCTIONS[opcode as usize]
    }
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
    ([ADC $l:tt]) => {
        Instruction::ADC(AM!($l))
    };
    ([ADD $l:tt]) => {
        Instruction::ADD(AM!($l))
    };
    ([ADDHL $l:tt]) => {
        Instruction::ADDHL(R!($l))
    };
    ([JR $c:tt]) => {
        Instruction::JR(C!($c))
    };
    ([SBC $l:tt]) => {
        Instruction::SBC(AM!($l))
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
}

static INSTRUCTIONS: [Instruction; 0x100] = INST_MAP![
/*          x0              x1              x2              x3              x4              x5              x6              x7              x8              x9              xA              xB              xC              xD              xE              xF  */
/* 0x */    NOP;            [LD BC, d16];   [LD (BC), A];   [INC BC];       [INC B];        [DEC B];        [LD B, d8];     RLCA;           [LD a16, SP];   [ADDHL BC];     [LD A, (BC)];   [DEC BC];       [INC C];        [DEC C];        [LD C, d8];     RRCA;
/* 1x */    STOP;           [LD DE, d16];   [LD (DE), A];   [INC DE];       [INC D];        [DEC D];        [LD D, d8];     RLA;            [JR None];      [ADDHL DE];     [LD A, (DE)];   [DEC DE];       [INC E];        [DEC E];        [LD E, d8];     RRA;
/* 2x */    [JR NZ];        [LD HL, d16];   LDI1;           [INC HL];       [INC H];        [DEC H];        [LD H, d8];     DAA;            [JR Z];         [ADDHL HL];     LDI2;           [DEC HL];       [INC L];        [DEC L];        [LD L, d8];     CPL;
/* 3x */    [JR NC];        [LD SP, d16];   LDD1;           [INC SP];       INCHL;          DECHL;          [LD (HL), d8];  SCF;            [JR C];         [ADDHL SP];     LDD2;           [DEC SP];       [INC A];        [DEC A];        [LD A, d8];     CCF;
/* 4x */    [LD B, B];      [LD B, C];      [LD B, D];      [LD B, E];      [LD B, H];      [LD B, L];      [LD B, (HL)];   [LD B, A];      [LD C, B];      [LD C, C];      [LD C, D];      [LD C, E];      [LD C, H];      [LD C, L];      [LD C, (HL)];   [LD C, A];
/* 5x */    [LD D, B];      [LD D, C];      [LD D, D];      [LD D, E];      [LD D, H];      [LD D, L];      [LD D, (HL)];   [LD D, A];      [LD E, B];      [LD E, C];      [LD E, D];      [LD E, E];      [LD E, H];      [LD E, L];      [LD E, (HL)];   [LD E, A];
/* 6x */    [LD H, B];      [LD H, C];      [LD H, D];      [LD H, E];      [LD H, H];      [LD H, L];      [LD H, (HL)];   [LD H, A];      [LD L, B];      [LD L, C];      [LD L, D];      [LD L, E];      [LD L, H];      [LD L, L];      [LD L, (HL)];   [LD L, A];
/* 7x */    [LD (HL), B];   [LD (HL), C];   [LD (HL), D];   [LD (HL), E];   [LD (HL), H];   [LD (HL), L];   HALT;           [LD (HL), A];   [LD A, B];      [LD A, C];      [LD A, D];      [LD A, E];      [LD A, H];      [LD A, L];      [LD A, (HL)];   [LD A, A];
/* 8x */    [ADD B];        [ADD C];        [ADD D];        [ADD E];        [ADD H];        [ADD L];        [ADD (HL)];     [ADD A];        [ADC B];        [ADC C];        [ADC D];        [ADC E];        [ADC H];        [ADC L];        [ADC (HL)];     [ADC A];
/* 9x */    [SUB B];        [SUB C];        [SUB D];        [SUB E];        [SUB H];        [SUB L];        [SUB (HL)];     [SUB A];        [SBC B];        [SBC C];        [SBC D];        [SBC E];        [SBC H];        [SBC L];        [SBC (HL)];     [SBC A];
/* Ax */    [AND B];        [AND C];        [AND D];        [AND E];        [AND H];        [AND L];        [AND (HL)];     [AND A];        [XOR B];        [XOR C];        [XOR D];        [XOR E];        [XOR H];        [XOR L];        [XOR (HL)];     [XOR A];
/* Bx */    [OR B];         [OR C];         [OR D];         [OR E];         [OR H];         [OR L];         [OR (HL)];      [OR A];         [CP B];         [CP C];         [CP D];         [CP E];         [CP H];         [CP L];         [CP (HL)];      [CP A];
/* Cx */    [RET NZ];       [POP BC];       [JP NZ];        [JP None];      [CALL NZ];      [PUSH BC];      [ADD d8];       [RST 0x00];     [RET Z];        [RET None];     [JP Z];         PREFIX;         [CALL Z];       [CALL None];    [ADC d8];       [RST 0x08];
/* Dx */    [RET NC];       [POP DE];       [JP NC];        None;           [CALL NC];      [PUSH DE];      [SUB d8];       [RST 0x10];     [RET C];        RETI;           [JP C];         None;           [CALL C];       None;           [SBC d8];       [RST 0x18];
/* Ex */    [LD a8, A];     [POP HL];       [LD (C), A];    None;           None;           [PUSH HL];      [AND d8];       [RST 0x20];     ADDSP;          JPHL;           [LD a16, A];    None;           None;           None;           [XOR d8];       [RST 0x28];
/* Fx */    [LD A, a8];     POPAF;          [LD A, (C)];    DI;             None;           [PUSH AF];      [OR d8];        [RST 0x30];     LDHL;           [LD SP, HL];    [LD A, a16];    EI;             None;           None;           [CP d8];        [RST 0x38];
];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CBInstruction {
    None,
    BIT(u8, Register),
    RES(u8, Register),
    RL(Register),
    RLC(Register),
    RR(Register),
    RRC(Register),
    SET(u8, Register),
    SLA(Register),
    SRA(Register),
    SRL(Register),
    SWAP(Register),
}

macro_rules! CB_INST {
    ([$i:tt $l:tt,$r:tt]) => {
        CBInstruction::$i($l, R!($r))
    };
    ([$i:tt $l:tt]) => {
        CBInstruction::$i(R!($l))
    };
}

macro_rules! CB_INST_MAP {
    ($($val:tt);* $(;)?) => {{
        let mut a = [CBInstruction::None; 0x100];
        let mut idx = 0;
        $(
            a[idx] = CB_INST!($val);
            idx += 1;
        )*
        a
    }};
}

static CB_INSTRUCTIONS: [CBInstruction; 0x100] = CB_INST_MAP![
/*          x0              x1              x2              x3              x4              x5              x6              x7              x8              x9              xA              xB              xC              xD              xE              xF  */
/* 0x */    [RLC B];        [RLC C];        [RLC D];        [RLC E];        [RLC H];        [RLC L];        [RLC HL];       [RLC A];        [RRC B];        [RRC C];        [RRC D];        [RRC E];        [RRC H];        [RRC L];        [RRC HL];       [RRC A];
/* 1x */    [RL B];         [RL C];         [RL D];         [RL E];         [RL H];         [RL L];         [RL HL];        [RL A];         [RR B];         [RR C];         [RR D];         [RR E];         [RR H];         [RR L];         [RR HL];        [RR A];
/* 2x */    [SLA B];        [SLA C];        [SLA D];        [SLA E];        [SLA H];        [SLA L];        [SLA HL];       [SLA A];        [SRA B];        [SRA C];        [SRA D];        [SRA E];        [SRA H];        [SRA L];        [SRA HL];       [SRA A];
/* 3x */    [SWAP B];       [SWAP C];       [SWAP D];       [SWAP E];       [SWAP H];       [SWAP L];       [SWAP HL];      [SWAP A];       [SRL B];        [SRL C];        [SRL D];        [SRL E];        [SRL H];        [SRL L];        [SRL HL];       [SRL A];
/* 4x */    [BIT 0,B];      [BIT 0,C];      [BIT 0,D];      [BIT 0,E];      [BIT 0,H];      [BIT 0,L];      [BIT 0,HL];     [BIT 0,A];      [BIT 1,B];      [BIT 1,C];      [BIT 1,D];      [BIT 1,E];      [BIT 1,H];      [BIT 1,L];      [BIT 1,HL];     [BIT 1,A];
/* 5x */    [BIT 2,B];      [BIT 2,C];      [BIT 2,D];      [BIT 2,E];      [BIT 2,H];      [BIT 2,L];      [BIT 2,HL];     [BIT 2,A];      [BIT 3,B];      [BIT 3,C];      [BIT 3,D];      [BIT 3,E];      [BIT 3,H];      [BIT 3,L];      [BIT 3,HL];     [BIT 3,A];
/* 6x */    [BIT 4,B];      [BIT 4,C];      [BIT 4,D];      [BIT 4,E];      [BIT 4,H];      [BIT 4,L];      [BIT 4,HL];     [BIT 4,A];      [BIT 5,B];      [BIT 5,C];      [BIT 5,D];      [BIT 5,E];      [BIT 5,H];      [BIT 5,L];      [BIT 5,HL];     [BIT 5,A];
/* 7x */    [BIT 6,B];      [BIT 6,C];      [BIT 6,D];      [BIT 6,E];      [BIT 6,H];      [BIT 6,L];      [BIT 6,HL];     [BIT 6,A];      [BIT 7,B];      [BIT 7,C];      [BIT 7,D];      [BIT 7,E];      [BIT 7,H];      [BIT 7,L];      [BIT 7,HL];     [BIT 7,A];
/* 8x */    [RES 0,B];      [RES 0,C];      [RES 0,D];      [RES 0,E];      [RES 0,H];      [RES 0,L];      [RES 0,HL];     [RES 0,A];      [RES 1,B];      [RES 1,C];      [RES 1,D];      [RES 1,E];      [RES 1,H];      [RES 1,L];      [RES 1,HL];     [RES 1,A];
/* 9x */    [RES 2,B];      [RES 2,C];      [RES 2,D];      [RES 2,E];      [RES 2,H];      [RES 2,L];      [RES 2,HL];     [RES 2,A];      [RES 3,B];      [RES 3,C];      [RES 3,D];      [RES 3,E];      [RES 3,H];      [RES 3,L];      [RES 3,HL];     [RES 3,A];
/* Ax */    [RES 4,B];      [RES 4,C];      [RES 4,D];      [RES 4,E];      [RES 4,H];      [RES 4,L];      [RES 4,HL];     [RES 4,A];      [RES 5,B];      [RES 5,C];      [RES 5,D];      [RES 5,E];      [RES 5,H];      [RES 5,L];      [RES 5,HL];     [RES 5,A];
/* Bx */    [RES 6,B];      [RES 6,C];      [RES 6,D];      [RES 6,E];      [RES 6,H];      [RES 6,L];      [RES 6,HL];     [RES 6,A];      [RES 7,B];      [RES 7,C];      [RES 7,D];      [RES 7,E];      [RES 7,H];      [RES 7,L];      [RES 7,HL];     [RES 7,A];
/* Cx */    [SET 0,B];      [SET 0,C];      [SET 0,D];      [SET 0,E];      [SET 0,H];      [SET 0,L];      [SET 0,HL];     [SET 0,A];      [SET 1,B];      [SET 1,C];      [SET 1,D];      [SET 1,E];      [SET 1,H];      [SET 1,L];      [SET 1,HL];     [SET 1,A];
/* Dx */    [SET 2,B];      [SET 2,C];      [SET 2,D];      [SET 2,E];      [SET 2,H];      [SET 2,L];      [SET 2,HL];     [SET 2,A];      [SET 3,B];      [SET 3,C];      [SET 3,D];      [SET 3,E];      [SET 3,H];      [SET 3,L];      [SET 3,HL];     [SET 3,A];
/* Ex */    [SET 4,B];      [SET 4,C];      [SET 4,D];      [SET 4,E];      [SET 4,H];      [SET 4,L];      [SET 4,HL];     [SET 4,A];      [SET 5,B];      [SET 5,C];      [SET 5,D];      [SET 5,E];      [SET 5,H];      [SET 5,L];      [SET 5,HL];     [SET 5,A];
/* Fx */    [SET 6,B];      [SET 6,C];      [SET 6,D];      [SET 6,E];      [SET 6,H];      [SET 6,L];      [SET 6,HL];     [SET 6,A];      [SET 7,B];      [SET 7,C];      [SET 7,D];      [SET 7,E];      [SET 7,H];      [SET 7,L];      [SET 7,HL];     [SET 7,A];
];

impl CBInstruction {
    pub fn from(opcode: u8) -> &'static Self {
        &CB_INSTRUCTIONS[opcode as usize]
    }
}