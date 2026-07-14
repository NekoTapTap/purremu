use crate::cpu::Cpu;
use crate::cpu::registers::{CpuReg8, CpuReg16};

#[derive(PartialEq, Debug, Clone, Copy)]
pub(crate) enum CpuCondition {
    NZ,
    Z,
    NC,
    C,
    None,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub(crate) enum CpuCbOperand {
    Register(CpuReg8),
    HlMem,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub(crate) enum CpuAluOperation {
    Add,
    Adc,
    Sub,
    Sbc,
    And,
    Xor,
    Or,
    Cp,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub(crate) enum CpuCbOperation {
    Rlc,
    Rrc,
    Rl,
    Rr,
    Sla,
    Sra,
    Swap,
    Srl,
    Bit(u8),
    Res(u8),
    Set(u8),
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub(crate) struct CpuCbInstruction {
    pub(crate) operation: CpuCbOperation,
    pub(crate) operand: CpuCbOperand,
}

impl CpuCbInstruction {
    pub(crate) fn decode(opcode: u8) -> Self {
        let operand = match opcode & 0b111 {
            0 => CpuCbOperand::Register(CpuReg8::B),
            1 => CpuCbOperand::Register(CpuReg8::C),
            2 => CpuCbOperand::Register(CpuReg8::D),
            3 => CpuCbOperand::Register(CpuReg8::E),
            4 => CpuCbOperand::Register(CpuReg8::H),
            5 => CpuCbOperand::Register(CpuReg8::L),
            6 => CpuCbOperand::HlMem,
            7 => CpuCbOperand::Register(CpuReg8::A),
            _ => unreachable!(),
        };
        let operation_index = (opcode >> 3) & 0b111;
        let operation = match opcode >> 6 {
            0 => match operation_index {
                0 => CpuCbOperation::Rlc,
                1 => CpuCbOperation::Rrc,
                2 => CpuCbOperation::Rl,
                3 => CpuCbOperation::Rr,
                4 => CpuCbOperation::Sla,
                5 => CpuCbOperation::Sra,
                6 => CpuCbOperation::Swap,
                7 => CpuCbOperation::Srl,
                _ => unreachable!(),
            },
            1 => CpuCbOperation::Bit(operation_index),
            2 => CpuCbOperation::Res(operation_index),
            3 => CpuCbOperation::Set(operation_index),
            _ => unreachable!(),
        };

        Self { operation, operand }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
// TODO: use another model to represent instructions, because this model represents too much operations but SM83 don't have that many operations.
pub(crate) enum CpuInstruction {
    LdR8Imm8(CpuReg8),
    LdR8R8(CpuReg8, CpuReg8),
    LdR16Imm16(CpuReg16),
    LdR16memA(CpuReg16),
    LdAR16mem(CpuReg16),
    LdHlMemR8(CpuReg8),
    LdHlMemImm8,
    LdSpImm16,
    LdAddrA,
    LdAAddr,
    LdAddrSp,
    LdhA8A,
    LdhAA8,
    LdhCA,
    LdhAC,
    LdHlSpE8,
    LdSpHl,
    LdR8HlMem(CpuReg8),

    LdAHlIncMem,
    LdAHlDecMem,
    LdHlIncMemA,
    LdHlDecMemA,

    IncR8(CpuReg8),
    IncR16(CpuReg16),
    DecR8(CpuReg8),
    DecR16(CpuReg16),

    IncHlMem,
    DecHlMem,

    AddAImm8,
    AddAR8(CpuReg8),
    AddHlR16(CpuReg16),
    AdcAImm8,
    AdcAR8(CpuReg8),
    AddSpE8,

    SubAImm8,
    SubAR8(CpuReg8),
    SbcAImm8,
    SbcAR8(CpuReg8),

    AndAImm8,
    AndAR8(CpuReg8),
    OrAImm8,
    OrAR8(CpuReg8),
    XorAImm8,
    XorAR8(CpuReg8),
    AluAHlMem(CpuAluOperation),

    PopR16(CpuReg16),
    PushR16(CpuReg16), // special, for AF, it saves the flags register and accumulator register to the stack

    CpAImm8,
    CpAR8(CpuReg8),

    Illegal,

    JrNzE8,
    JrZE8,
    JrNcE8,
    JrCE8,
    JrE8,

    JpNzA16,
    JpZA16,
    JpNcA16,
    JpCA16,
    JpA16,
    JpHl,

    CallA16,
    CallNzA16,
    CallZA16,
    CallNcA16,
    CallCA16,

    Rst(u16), // quick call to a fixed address, save rom size and time.

    DI,
    EI,

    Ret(CpuCondition),
    RetI,

    PrefixCb,

    Rlca,
    Rrca,
    Rla,
    Rra,
    Daa,
    Cpl,
    Scf,
    Ccf,

    Halt,
    Stop,

    Nop,
}

impl Cpu {
    #[rustfmt::skip]
    pub(crate) fn initialize_instruction_set() -> [[CpuInstruction; 16]; 16] {
        use CpuInstruction::*;
        use crate::cpu::registers::CpuReg8::*;
        use crate::cpu::registers::CpuReg16::*;
        use CpuAluOperation::*;
        use CpuCondition::{NZ, Z, NC, C as CondC, None};

        [   // x0       x1              x2             x3            x4            x5            x6            x7            x8           x9            xA             xB          xC            xD           xE           xF
    /* 0x */[Nop        , LdR16Imm16(BC), LdR16memA(BC), IncR16(BC)  , IncR8(B)    , DecR8(B)    , LdR8Imm8(B) , Rlca        , LdAddrSp   , AddHlR16(BC), LdAR16mem(BC), DecR16(BC), IncR8(C)    , DecR8(C)   , LdR8Imm8(C) , Rrca       ],
    /* 1x */[Stop       , LdR16Imm16(DE), LdR16memA(DE), IncR16(DE)  , IncR8(D)    , DecR8(D)    , LdR8Imm8(D) , Rla         , JrE8       , AddHlR16(DE), LdAR16mem(DE), DecR16(DE), IncR8(E)    , DecR8(E)   , LdR8Imm8(E) , Rra        ],
    /* 2x */[JrNzE8     , LdR16Imm16(HL), LdHlIncMemA  , IncR16(HL)  , IncR8(H)    , DecR8(H)    , LdR8Imm8(H) , Daa         , JrZE8      , AddHlR16(HL), LdAHlIncMem  , DecR16(HL), IncR8(L)    , DecR8(L)   , LdR8Imm8(L) , Cpl        ],
    /* 3x */[JrNcE8     , LdSpImm16     , LdHlDecMemA  , IncR16(SP)  , IncHlMem    , DecHlMem    , LdHlMemImm8 , Scf         , JrCE8      , AddHlR16(SP), LdAHlDecMem  , DecR16(SP), IncR8(A)    , DecR8(A)   , LdR8Imm8(A) , Ccf        ],

    /* 4x */[LdR8R8(B,B), LdR8R8(B,C)  , LdR8R8(B,D)   , LdR8R8(B,E) , LdR8R8(B,H) , LdR8R8(B,L) , LdR8HlMem(B), LdR8R8(B,A) , LdR8R8(C,B), LdR8R8(C,C) , LdR8R8(C,D)  , LdR8R8(C,E), LdR8R8(C,H), LdR8R8(C,L), LdR8HlMem(C), LdR8R8(C,A)],
    /* 5x */[LdR8R8(D,B), LdR8R8(D,C)  , LdR8R8(D,D)   , LdR8R8(D,E) , LdR8R8(D,H) , LdR8R8(D,L) , LdR8HlMem(D), LdR8R8(D,A) , LdR8R8(E,B), LdR8R8(E,C) , LdR8R8(E,D)  , LdR8R8(E,E), LdR8R8(E,H), LdR8R8(E,L), LdR8HlMem(E), LdR8R8(E,A)],
    /* 6x */[LdR8R8(H,B), LdR8R8(H,C)  , LdR8R8(H,D)   , LdR8R8(H,E) , LdR8R8(H,H) , LdR8R8(H,L) , LdR8HlMem(H), LdR8R8(H,A) , LdR8R8(L,B), LdR8R8(L,C) , LdR8R8(L,D)  , LdR8R8(L,E), LdR8R8(L,H), LdR8R8(L,L), LdR8HlMem(L), LdR8R8(L,A)],
    /* 7x */[LdHlMemR8(B), LdHlMemR8(C), LdHlMemR8(D)  , LdHlMemR8(E), LdHlMemR8(H), LdHlMemR8(L), Halt        , LdHlMemR8(A), LdR8R8(A,B), LdR8R8(A,C) , LdR8R8(A,D)  , LdR8R8(A,E), LdR8R8(A,H), LdR8R8(A,L), LdR8HlMem(A), LdR8R8(A,A)],

    /* 8x */[AddAR8(B)  , AddAR8(C)    , AddAR8(D)     , AddAR8(E)   , AddAR8(H)   , AddAR8(L)   , AluAHlMem(Add), AddAR8(A), AdcAR8(B)  , AdcAR8(C)    , AdcAR8(D)    , AdcAR8(E)  , AdcAR8(H)  , AdcAR8(L)  , AluAHlMem(Adc), AdcAR8(A)],
    /* 9x */[SubAR8(B)  , SubAR8(C)    , SubAR8(D)     , SubAR8(E)   , SubAR8(H)   , SubAR8(L)   , AluAHlMem(Sub), SubAR8(A), SbcAR8(B)  , SbcAR8(C)    , SbcAR8(D)    , SbcAR8(E)  , SbcAR8(H)  , SbcAR8(L)  , AluAHlMem(Sbc), SbcAR8(A)],
    /* Ax */[AndAR8(B)  , AndAR8(C)    , AndAR8(D)     , AndAR8(E)   , AndAR8(H)   , AndAR8(L)   , AluAHlMem(And), AndAR8(A), XorAR8(B)  , XorAR8(C)    , XorAR8(D)    , XorAR8(E)  , XorAR8(H)  , XorAR8(L)  , AluAHlMem(Xor), XorAR8(A)],
    /* Bx */[OrAR8(B)   , OrAR8(C)     , OrAR8(D)      , OrAR8(E)    , OrAR8(H)    , OrAR8(L)    , AluAHlMem(Or), OrAR8(A)  , CpAR8(B)   , CpAR8(C)     , CpAR8(D)     , CpAR8(E)   , CpAR8(H)   , CpAR8(L)   , AluAHlMem(Cp), CpAR8(A)],

    /* Cx */[Ret(NZ)    , PopR16(BC)   , JpNzA16       , JpA16       , CallNzA16   , PushR16(BC) , AddAImm8   , Rst(0x00)   , Ret(Z)     , Ret(None)     , JpZA16      , PrefixCb   , CallZA16   , CallA16    , AdcAImm8   , Rst(0x08)  ],
    /* Dx */[Ret(NC)    , PopR16(DE)   , JpNcA16       , Illegal     , CallNcA16   , PushR16(DE) , SubAImm8   , Rst(0x10)   , Ret(CondC) , RetI          , JpCA16      , Illegal    , CallCA16   , Illegal    , SbcAImm8   , Rst(0x18)  ],
    /* Ex */[LdhA8A     , PopR16(HL)   , LdhCA         , Illegal     , Illegal     , PushR16(HL) , AndAImm8   , Rst(0x20)   , AddSpE8    , JpHl          , LdAddrA     , Illegal    , Illegal    , Illegal    , XorAImm8   , Rst(0x28)  ],
    /* Fx */[LdhAA8     , PopR16(AF)   , LdhAC         , DI          , Illegal     , PushR16(AF) , OrAImm8    , Rst(0x30)   , LdHlSpE8   , LdSpHl        , LdAAddr     , EI         , Illegal    , Illegal    , CpAImm8    , Rst(0x38)  ],
        ]
    }
}
