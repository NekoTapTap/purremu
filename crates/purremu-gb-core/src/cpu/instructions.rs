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
    LdhA8A,
    LdhAA8,
    LdhCA,
    LdhAC,

    IncR8(CpuReg8),
    IncR16(CpuReg16),
    DecR8(CpuReg8),
    DecR16(CpuReg16),

    AddAImm8,
    AddAR8(CpuReg8),
    AddHlR16(CpuReg16),
    AdcAImm8,
    AdcAR8(CpuReg8),

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
    XorAHlMem,

    PopR16(CpuReg16),
    PushR16(CpuReg16),
    PopAF,
    PushAF,

    CpAImm8,
    CpAR8(CpuReg8),

    Illegal,
    NoImpl,

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

    DI,
    EI,

    Ret(CpuCondition),
    RetI,

    Nop,
}

impl Cpu {
    #[rustfmt::skip]
    pub(crate) fn initialize_instruction_set() -> [[CpuInstruction; 16]; 16] {
        use CpuInstruction::*;
        use crate::cpu::registers::CpuReg8::*;
        use crate::cpu::registers::CpuReg16::*;
        use CpuCondition::{NZ, Z, NC, C as CondC, None};

        [   // x0       x1              x2             x3           x4           x5           x6           x7           x8           x9            xA             xB          xC            xD           xE           xF
    /* 0x */[Nop        , LdR16Imm16(BC), LdR16memA(BC), IncR16(BC), IncR8(B)   , DecR8(B)   , LdR8Imm8(B), NoImpl     , NoImpl     , AddHlR16(BC), LdAR16mem(BC), DecR16(BC), IncR8(C)    , DecR8(C)   , LdR8Imm8(C), NoImpl     ],
    /* 1x */[NoImpl     , LdR16Imm16(DE), LdR16memA(DE), IncR16(DE), IncR8(D)   , DecR8(D)   , LdR8Imm8(D), NoImpl     , JrE8       , AddHlR16(DE), LdAR16mem(DE), DecR16(DE), IncR8(E)    , DecR8(E)   , LdR8Imm8(E), NoImpl     ],
    /* 2x */[JrNzE8     , LdR16Imm16(HL), LdR16memA(HL), IncR16(HL), IncR8(H)   , DecR8(H)   , LdR8Imm8(H), NoImpl     , JrZE8      , AddHlR16(HL), LdAR16mem(HL), DecR16(HL), IncR8(L)    , DecR8(L)   , LdR8Imm8(L), NoImpl     ],
    /* 3x */[JrNcE8     , LdSpImm16     , NoImpl        , NoImpl     , NoImpl     , NoImpl     , LdHlMemImm8, NoImpl     , JrCE8      , NoImpl      , NoImpl       , NoImpl    , IncR8(A)    , DecR8(A)   , LdR8Imm8(A), NoImpl     ],

    /* 4x */[LdR8R8(B,B), LdR8R8(B,C)  , LdR8R8(B,D)   , LdR8R8(B,E), LdR8R8(B,H), LdR8R8(B,L), NoImpl   , LdR8R8(B,A)  , LdR8R8(C,B), LdR8R8(C,C) , LdR8R8(C,D)  , LdR8R8(C,E), LdR8R8(C,H), LdR8R8(C,L), NoImpl     , LdR8R8(C,A)],
    /* 5x */[LdR8R8(D,B), LdR8R8(D,C)  , LdR8R8(D,D)   , LdR8R8(D,E), LdR8R8(D,H), LdR8R8(D,L), NoImpl   , LdR8R8(D,A)  , LdR8R8(E,B), LdR8R8(E,C) , LdR8R8(E,D)  , LdR8R8(E,E), LdR8R8(E,H), LdR8R8(E,L), NoImpl     , LdR8R8(E,A)],
    /* 6x */[LdR8R8(H,B), LdR8R8(H,C)  , LdR8R8(H,D)   , LdR8R8(H,E), LdR8R8(H,H), LdR8R8(H,L), NoImpl   , LdR8R8(H,A)  , LdR8R8(L,B), LdR8R8(L,C) , LdR8R8(L,D)  , LdR8R8(L,E), LdR8R8(L,H), LdR8R8(L,L), NoImpl     , LdR8R8(L,A)],
    /* 7x */[LdHlMemR8(B), LdHlMemR8(C), LdHlMemR8(D), LdHlMemR8(E) , LdHlMemR8(H),LdHlMemR8(L), NoImpl  , LdHlMemR8(A) , LdR8R8(A,B), LdR8R8(A,C) , LdR8R8(A,D)  , LdR8R8(A,E), LdR8R8(A,H), LdR8R8(A,L), NoImpl     , LdR8R8(A,A)],

    /* 8x */[AddAR8(B)  , AddAR8(C)    , AddAR8(D)     , AddAR8(E)  , AddAR8(H)  , AddAR8(L)  , NoImpl   , AddAR8(A)   , AdcAR8(B)  , AdcAR8(C)    , AdcAR8(D)    , AdcAR8(E)  , AdcAR8(H)  , AdcAR8(L)  , NoImpl     , AdcAR8(A)  ],
    /* 9x */[SubAR8(B)  , SubAR8(C)    , SubAR8(D)     , SubAR8(E)  , SubAR8(H)  , SubAR8(L)  , NoImpl   , SubAR8(A)   , SbcAR8(B)  , SbcAR8(C)    , SbcAR8(D)    , SbcAR8(E)  , SbcAR8(H)  , SbcAR8(L)  , NoImpl     , SbcAR8(A)  ],
    /* Ax */[AndAR8(B)  , AndAR8(C)    , AndAR8(D)     , AndAR8(E)  , AndAR8(H)  , AndAR8(L)  , NoImpl   , AndAR8(A)   , XorAR8(B)  , XorAR8(C)    , XorAR8(D)    , XorAR8(E)  , XorAR8(H)  , XorAR8(L)  , XorAHlMem  , XorAR8(A)  ],
    /* Bx */[OrAR8(B)   , OrAR8(C)     , OrAR8(D)      , OrAR8(E)   , OrAR8(H)   , OrAR8(L)   , NoImpl   , OrAR8(A)    , CpAR8(B)   , CpAR8(C)     , CpAR8(D)     , CpAR8(E)   , CpAR8(H)   , CpAR8(L)   , NoImpl     , CpAR8(A)   ],

    /* Cx */[Ret(NZ)    , PopR16(BC)   , JpNzA16       , JpA16      , CallNzA16  , PushR16(BC), AddAImm8 , NoImpl      , Ret(Z)     , Ret(None)     , JpZA16      , NoImpl     , CallZA16   , CallA16    , AdcAImm8   , NoImpl     ],
    /* Dx */[Ret(NC)    , PopR16(DE)   , JpNcA16       , Illegal    , CallNcA16  , PushR16(DE), SubAImm8 , NoImpl      , Ret(CondC) , RetI          , JpCA16      , Illegal    , CallCA16   , Illegal    , SbcAImm8   , NoImpl     ],
    /* Ex */[LdhA8A     , PopR16(HL)   , LdhCA        , Illegal    , Illegal    , PushR16(HL), AndAImm8 , NoImpl      , NoImpl     , JpHl          , LdAddrA     , Illegal    , Illegal    , Illegal    , XorAImm8   , NoImpl     ],
    /* Fx */[LdhAA8     , PopAF        , LdhAC        , DI         , Illegal    , PushAF     , OrAImm8  , NoImpl      , NoImpl     , NoImpl        , LdAAddr     , EI         , Illegal    , Illegal    , CpAImm8    , NoImpl     ],
        ]
    }
}
