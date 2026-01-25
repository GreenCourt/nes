#[derive(Debug, PartialEq)]
#[allow(non_camel_case_types)]
pub enum AddressingMode {
    Accumulator,
    Immediate,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Indirect,
    IndirectX,
    IndirectY,
    Relative,
    Implied,
}

#[derive(Debug, PartialEq)]
pub enum Mnemonic {
    ADC,
    AND,
    ASL,
    BCC,
    BCS,
    BEQ,
    BIT,
    BMI,
    BNE,
    BPL,
    BRK,
    BVC,
    BVS,
    CLC,
    CLD,
    CLI,
    CLV,
    CMP,
    CPX,
    CPY,
    DEC,
    DEX,
    DEY,
    EOR,
    INC,
    INX,
    INY,
    JMP,
    JSR,
    LDA,
    LDX,
    LDY,
    LSR,
    NOP,
    ORA,
    PHA,
    PHP,
    PLA,
    PLP,
    ROL,
    ROR,
    RTI,
    RTS,
    SBC,
    SEC,
    SED,
    SEI,
    STA,
    STX,
    STY,
    TAX,
    TAY,
    TSX,
    TXA,
    TXS,
    TYA,
    // --- unofficial ---
    DCP,
    ISB,
    LAX,
    RLA,
    RRA,
    SAX,
    SLO,
    SRE,
    Unknown,
}

pub struct Instruction {
    pub mnemonic: Mnemonic,
    pub addressing_mode: AddressingMode,
}

pub const INSTRUCTIONS: &[Instruction] = &[
    // 0x00
    Instruction {
        mnemonic: Mnemonic::BRK,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x01
    Instruction {
        mnemonic: Mnemonic::ORA,
        addressing_mode: AddressingMode::IndirectX,
    },
    // 0x02
    Instruction {
        mnemonic: Mnemonic::Unknown,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x03
    Instruction {
        mnemonic: Mnemonic::SLO,
        addressing_mode: AddressingMode::IndirectX,
    },
    // 0x04
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0x05
    Instruction {
        mnemonic: Mnemonic::ORA,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0x06
    Instruction {
        mnemonic: Mnemonic::ASL,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0x07
    Instruction {
        mnemonic: Mnemonic::SLO,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0x08
    Instruction {
        mnemonic: Mnemonic::PHP,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x09
    Instruction {
        mnemonic: Mnemonic::ORA,
        addressing_mode: AddressingMode::Immediate,
    },
    // 0x0A
    Instruction {
        mnemonic: Mnemonic::ASL,
        addressing_mode: AddressingMode::Accumulator,
    },
    // 0x0B
    Instruction {
        mnemonic: Mnemonic::Unknown,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x0C
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0x0D
    Instruction {
        mnemonic: Mnemonic::ORA,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0x0E
    Instruction {
        mnemonic: Mnemonic::ASL,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0x0F
    Instruction {
        mnemonic: Mnemonic::SLO,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0x10
    Instruction {
        mnemonic: Mnemonic::BPL,
        addressing_mode: AddressingMode::Relative,
    },
    // 0x11
    Instruction {
        mnemonic: Mnemonic::ORA,
        addressing_mode: AddressingMode::IndirectY,
    },
    // 0x12
    Instruction {
        mnemonic: Mnemonic::Unknown,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x13
    Instruction {
        mnemonic: Mnemonic::SLO,
        addressing_mode: AddressingMode::IndirectY,
    },
    // 0x14
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0x15
    Instruction {
        mnemonic: Mnemonic::ORA,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0x16
    Instruction {
        mnemonic: Mnemonic::ASL,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0x17
    Instruction {
        mnemonic: Mnemonic::SLO,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0x18
    Instruction {
        mnemonic: Mnemonic::CLC,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x19
    Instruction {
        mnemonic: Mnemonic::ORA,
        addressing_mode: AddressingMode::AbsoluteY,
    },
    // 0x1A
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x1B
    Instruction {
        mnemonic: Mnemonic::SLO,
        addressing_mode: AddressingMode::AbsoluteY,
    },
    // 0x1C
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::AbsoluteX,
    },
    // 0x1D
    Instruction {
        mnemonic: Mnemonic::ORA,
        addressing_mode: AddressingMode::AbsoluteX,
    },
    // 0x1E
    Instruction {
        mnemonic: Mnemonic::ASL,
        addressing_mode: AddressingMode::AbsoluteX,
    },
    // 0x1F
    Instruction {
        mnemonic: Mnemonic::SLO,
        addressing_mode: AddressingMode::AbsoluteX,
    },
    // 0x20
    Instruction {
        mnemonic: Mnemonic::JSR,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0x21
    Instruction {
        mnemonic: Mnemonic::AND,
        addressing_mode: AddressingMode::IndirectX,
    },
    // 0x22
    Instruction {
        mnemonic: Mnemonic::Unknown,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x23
    Instruction {
        mnemonic: Mnemonic::RLA,
        addressing_mode: AddressingMode::IndirectX,
    },
    // 0x24
    Instruction {
        mnemonic: Mnemonic::BIT,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0x25
    Instruction {
        mnemonic: Mnemonic::AND,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0x26
    Instruction {
        mnemonic: Mnemonic::ROL,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0x27
    Instruction {
        mnemonic: Mnemonic::RLA,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0x28
    Instruction {
        mnemonic: Mnemonic::PLP,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x29
    Instruction {
        mnemonic: Mnemonic::AND,
        addressing_mode: AddressingMode::Immediate,
    },
    // 0x2A
    Instruction {
        mnemonic: Mnemonic::ROL,
        addressing_mode: AddressingMode::Accumulator,
    },
    // 0x2B
    Instruction {
        mnemonic: Mnemonic::Unknown,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x2C
    Instruction {
        mnemonic: Mnemonic::BIT,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0x2D
    Instruction {
        mnemonic: Mnemonic::AND,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0x2E
    Instruction {
        mnemonic: Mnemonic::ROL,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0x2F
    Instruction {
        mnemonic: Mnemonic::RLA,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0x30
    Instruction {
        mnemonic: Mnemonic::BMI,
        addressing_mode: AddressingMode::Relative,
    },
    // 0x31
    Instruction {
        mnemonic: Mnemonic::AND,
        addressing_mode: AddressingMode::IndirectY,
    },
    // 0x32
    Instruction {
        mnemonic: Mnemonic::Unknown,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x33
    Instruction {
        mnemonic: Mnemonic::RLA,
        addressing_mode: AddressingMode::IndirectY,
    },
    // 0x34
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0x35
    Instruction {
        mnemonic: Mnemonic::AND,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0x36
    Instruction {
        mnemonic: Mnemonic::ROL,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0x37
    Instruction {
        mnemonic: Mnemonic::RLA,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0x38
    Instruction {
        mnemonic: Mnemonic::SEC,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x39
    Instruction {
        mnemonic: Mnemonic::AND,
        addressing_mode: AddressingMode::AbsoluteY,
    },
    // 0x3A
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x3B
    Instruction {
        mnemonic: Mnemonic::RLA,
        addressing_mode: AddressingMode::AbsoluteY,
    },
    // 0x3C
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::AbsoluteX,
    },
    // 0x3D
    Instruction {
        mnemonic: Mnemonic::AND,
        addressing_mode: AddressingMode::AbsoluteX,
    },
    // 0x3E
    Instruction {
        mnemonic: Mnemonic::ROL,
        addressing_mode: AddressingMode::AbsoluteX,
    },
    // 0x3F
    Instruction {
        mnemonic: Mnemonic::RLA,
        addressing_mode: AddressingMode::AbsoluteX,
    },
    // 0x40
    Instruction {
        mnemonic: Mnemonic::RTI,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x41
    Instruction {
        mnemonic: Mnemonic::EOR,
        addressing_mode: AddressingMode::IndirectX,
    },
    // 0x42
    Instruction {
        mnemonic: Mnemonic::Unknown,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x43
    Instruction {
        mnemonic: Mnemonic::SRE,
        addressing_mode: AddressingMode::IndirectX,
    },
    // 0x44
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0x45
    Instruction {
        mnemonic: Mnemonic::EOR,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0x46
    Instruction {
        mnemonic: Mnemonic::LSR,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0x47
    Instruction {
        mnemonic: Mnemonic::SRE,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0x48
    Instruction {
        mnemonic: Mnemonic::PHA,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x49
    Instruction {
        mnemonic: Mnemonic::EOR,
        addressing_mode: AddressingMode::Immediate,
    },
    // 0x4A
    Instruction {
        mnemonic: Mnemonic::LSR,
        addressing_mode: AddressingMode::Accumulator,
    },
    // 0x4B
    Instruction {
        mnemonic: Mnemonic::Unknown,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x4C
    Instruction {
        mnemonic: Mnemonic::JMP,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0x4D
    Instruction {
        mnemonic: Mnemonic::EOR,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0x4E
    Instruction {
        mnemonic: Mnemonic::LSR,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0x4F
    Instruction {
        mnemonic: Mnemonic::SRE,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0x50
    Instruction {
        mnemonic: Mnemonic::BVC,
        addressing_mode: AddressingMode::Relative,
    },
    // 0x51
    Instruction {
        mnemonic: Mnemonic::EOR,
        addressing_mode: AddressingMode::IndirectY,
    },
    // 0x52
    Instruction {
        mnemonic: Mnemonic::Unknown,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x53
    Instruction {
        mnemonic: Mnemonic::SRE,
        addressing_mode: AddressingMode::IndirectY,
    },
    // 0x54
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0x55
    Instruction {
        mnemonic: Mnemonic::EOR,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0x56
    Instruction {
        mnemonic: Mnemonic::LSR,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0x57
    Instruction {
        mnemonic: Mnemonic::SRE,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0x58
    Instruction {
        mnemonic: Mnemonic::CLI,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x59
    Instruction {
        mnemonic: Mnemonic::EOR,
        addressing_mode: AddressingMode::AbsoluteY,
    },
    // 0x5A
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x5B
    Instruction {
        mnemonic: Mnemonic::SRE,
        addressing_mode: AddressingMode::AbsoluteY,
    },
    // 0x5C
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::AbsoluteX,
    },
    // 0x5D
    Instruction {
        mnemonic: Mnemonic::EOR,
        addressing_mode: AddressingMode::AbsoluteX,
    },
    // 0x5E
    Instruction {
        mnemonic: Mnemonic::LSR,
        addressing_mode: AddressingMode::AbsoluteX,
    },
    // 0x5F
    Instruction {
        mnemonic: Mnemonic::SRE,
        addressing_mode: AddressingMode::AbsoluteX,
    },
    // 0x60
    Instruction {
        mnemonic: Mnemonic::RTS,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x61
    Instruction {
        mnemonic: Mnemonic::ADC,
        addressing_mode: AddressingMode::IndirectX,
    },
    // 0x62
    Instruction {
        mnemonic: Mnemonic::Unknown,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x63
    Instruction {
        mnemonic: Mnemonic::RRA,
        addressing_mode: AddressingMode::IndirectX,
    },
    // 0x64
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0x65
    Instruction {
        mnemonic: Mnemonic::ADC,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0x66
    Instruction {
        mnemonic: Mnemonic::ROR,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0x67
    Instruction {
        mnemonic: Mnemonic::RRA,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0x68
    Instruction {
        mnemonic: Mnemonic::PLA,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x69
    Instruction {
        mnemonic: Mnemonic::ADC,
        addressing_mode: AddressingMode::Immediate,
    },
    // 0x6A
    Instruction {
        mnemonic: Mnemonic::ROR,
        addressing_mode: AddressingMode::Accumulator,
    },
    // 0x6B
    Instruction {
        mnemonic: Mnemonic::Unknown,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x6C
    Instruction {
        mnemonic: Mnemonic::JMP,
        addressing_mode: AddressingMode::Indirect,
    },
    // 0x6D
    Instruction {
        mnemonic: Mnemonic::ADC,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0x6E
    Instruction {
        mnemonic: Mnemonic::ROR,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0x6F
    Instruction {
        mnemonic: Mnemonic::RRA,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0x70
    Instruction {
        mnemonic: Mnemonic::BVS,
        addressing_mode: AddressingMode::Relative,
    },
    // 0x71
    Instruction {
        mnemonic: Mnemonic::ADC,
        addressing_mode: AddressingMode::IndirectY,
    },
    // 0x72
    Instruction {
        mnemonic: Mnemonic::Unknown,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x73
    Instruction {
        mnemonic: Mnemonic::RRA,
        addressing_mode: AddressingMode::IndirectY,
    },
    // 0x74
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0x75
    Instruction {
        mnemonic: Mnemonic::ADC,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0x76
    Instruction {
        mnemonic: Mnemonic::ROR,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0x77
    Instruction {
        mnemonic: Mnemonic::RRA,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0x78
    Instruction {
        mnemonic: Mnemonic::SEI,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x79
    Instruction {
        mnemonic: Mnemonic::ADC,
        addressing_mode: AddressingMode::AbsoluteY,
    },
    // 0x7A
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x7B
    Instruction {
        mnemonic: Mnemonic::RRA,
        addressing_mode: AddressingMode::AbsoluteY,
    },
    // 0x7C
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::AbsoluteX,
    },
    // 0x7D
    Instruction {
        mnemonic: Mnemonic::ADC,
        addressing_mode: AddressingMode::AbsoluteX,
    },
    // 0x7E
    Instruction {
        mnemonic: Mnemonic::ROR,
        addressing_mode: AddressingMode::AbsoluteX,
    },
    // 0x7F
    Instruction {
        mnemonic: Mnemonic::RRA,
        addressing_mode: AddressingMode::AbsoluteX,
    },
    // 0x80
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::Immediate,
    },
    // 0x81
    Instruction {
        mnemonic: Mnemonic::STA,
        addressing_mode: AddressingMode::IndirectX,
    },
    // 0x82
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::Immediate,
    },
    // 0x83
    Instruction {
        mnemonic: Mnemonic::SAX,
        addressing_mode: AddressingMode::IndirectX,
    },
    // 0x84
    Instruction {
        mnemonic: Mnemonic::STY,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0x85
    Instruction {
        mnemonic: Mnemonic::STA,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0x86
    Instruction {
        mnemonic: Mnemonic::STX,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0x87
    Instruction {
        mnemonic: Mnemonic::SAX,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0x88
    Instruction {
        mnemonic: Mnemonic::DEY,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x89
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::Immediate,
    },
    // 0x8A
    Instruction {
        mnemonic: Mnemonic::TXA,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x8B
    Instruction {
        mnemonic: Mnemonic::Unknown,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x8C
    Instruction {
        mnemonic: Mnemonic::STY,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0x8D
    Instruction {
        mnemonic: Mnemonic::STA,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0x8E
    Instruction {
        mnemonic: Mnemonic::STX,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0x8F
    Instruction {
        mnemonic: Mnemonic::SAX,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0x90
    Instruction {
        mnemonic: Mnemonic::BCC,
        addressing_mode: AddressingMode::Relative,
    },
    // 0x91
    Instruction {
        mnemonic: Mnemonic::STA,
        addressing_mode: AddressingMode::IndirectY,
    },
    // 0x92
    Instruction {
        mnemonic: Mnemonic::Unknown,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x93
    Instruction {
        mnemonic: Mnemonic::Unknown,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x94
    Instruction {
        mnemonic: Mnemonic::STY,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0x95
    Instruction {
        mnemonic: Mnemonic::STA,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0x96
    Instruction {
        mnemonic: Mnemonic::STX,
        addressing_mode: AddressingMode::ZeroPageY,
    },
    // 0x97
    Instruction {
        mnemonic: Mnemonic::SAX,
        addressing_mode: AddressingMode::ZeroPageY,
    },
    // 0x98
    Instruction {
        mnemonic: Mnemonic::TYA,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x99
    Instruction {
        mnemonic: Mnemonic::STA,
        addressing_mode: AddressingMode::AbsoluteY,
    },
    // 0x9A
    Instruction {
        mnemonic: Mnemonic::TXS,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x9B
    Instruction {
        mnemonic: Mnemonic::Unknown,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x9C
    Instruction {
        mnemonic: Mnemonic::Unknown,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x9D
    Instruction {
        mnemonic: Mnemonic::STA,
        addressing_mode: AddressingMode::AbsoluteX,
    },
    // 0x9E
    Instruction {
        mnemonic: Mnemonic::Unknown,
        addressing_mode: AddressingMode::Implied,
    },
    // 0x9F
    Instruction {
        mnemonic: Mnemonic::Unknown,
        addressing_mode: AddressingMode::Implied,
    },
    // 0xA0
    Instruction {
        mnemonic: Mnemonic::LDY,
        addressing_mode: AddressingMode::Immediate,
    },
    // 0xA1
    Instruction {
        mnemonic: Mnemonic::LDA,
        addressing_mode: AddressingMode::IndirectX,
    },
    // 0xA2
    Instruction {
        mnemonic: Mnemonic::LDX,
        addressing_mode: AddressingMode::Immediate,
    },
    // 0xA3
    Instruction {
        mnemonic: Mnemonic::LAX,
        addressing_mode: AddressingMode::IndirectX,
    },
    // 0xA4
    Instruction {
        mnemonic: Mnemonic::LDY,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0xA5
    Instruction {
        mnemonic: Mnemonic::LDA,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0xA6
    Instruction {
        mnemonic: Mnemonic::LDX,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0xA7
    Instruction {
        mnemonic: Mnemonic::LAX,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0xA8
    Instruction {
        mnemonic: Mnemonic::TAY,
        addressing_mode: AddressingMode::Implied,
    },
    // 0xA9
    Instruction {
        mnemonic: Mnemonic::LDA,
        addressing_mode: AddressingMode::Immediate,
    },
    // 0xAA
    Instruction {
        mnemonic: Mnemonic::TAX,
        addressing_mode: AddressingMode::Implied,
    },
    // 0xAB
    Instruction {
        mnemonic: Mnemonic::Unknown,
        addressing_mode: AddressingMode::Implied,
    },
    // 0xAC
    Instruction {
        mnemonic: Mnemonic::LDY,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0xAD
    Instruction {
        mnemonic: Mnemonic::LDA,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0xAE
    Instruction {
        mnemonic: Mnemonic::LDX,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0xAF
    Instruction {
        mnemonic: Mnemonic::LAX,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0xB0
    Instruction {
        mnemonic: Mnemonic::BCS,
        addressing_mode: AddressingMode::Relative,
    },
    // 0xB1
    Instruction {
        mnemonic: Mnemonic::LDA,
        addressing_mode: AddressingMode::IndirectY,
    },
    // 0xB2
    Instruction {
        mnemonic: Mnemonic::Unknown,
        addressing_mode: AddressingMode::Implied,
    },
    // 0xB3
    Instruction {
        mnemonic: Mnemonic::LAX,
        addressing_mode: AddressingMode::IndirectY,
    },
    // 0xB4
    Instruction {
        mnemonic: Mnemonic::LDY,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0xB5
    Instruction {
        mnemonic: Mnemonic::LDA,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0xB6
    Instruction {
        mnemonic: Mnemonic::LDX,
        addressing_mode: AddressingMode::ZeroPageY,
    },
    // 0xB7
    Instruction {
        mnemonic: Mnemonic::LAX,
        addressing_mode: AddressingMode::ZeroPageY,
    },
    // 0xB8
    Instruction {
        mnemonic: Mnemonic::CLV,
        addressing_mode: AddressingMode::Implied,
    },
    // 0xB9
    Instruction {
        mnemonic: Mnemonic::LDA,
        addressing_mode: AddressingMode::AbsoluteY,
    },
    // 0xBA
    Instruction {
        mnemonic: Mnemonic::TSX,
        addressing_mode: AddressingMode::Implied,
    },
    // 0xBB
    Instruction {
        mnemonic: Mnemonic::Unknown,
        addressing_mode: AddressingMode::Implied,
    },
    // 0xBC
    Instruction {
        mnemonic: Mnemonic::LDY,
        addressing_mode: AddressingMode::AbsoluteX,
    },
    // 0xBD
    Instruction {
        mnemonic: Mnemonic::LDA,
        addressing_mode: AddressingMode::AbsoluteX,
    },
    // 0xBE
    Instruction {
        mnemonic: Mnemonic::LDX,
        addressing_mode: AddressingMode::AbsoluteY,
    },
    // 0xBF
    Instruction {
        mnemonic: Mnemonic::LAX,
        addressing_mode: AddressingMode::AbsoluteY,
    },
    // 0xC0
    Instruction {
        mnemonic: Mnemonic::CPY,
        addressing_mode: AddressingMode::Immediate,
    },
    // 0xC1
    Instruction {
        mnemonic: Mnemonic::CMP,
        addressing_mode: AddressingMode::IndirectX,
    },
    // 0xC2
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::Immediate,
    },
    // 0xC3
    Instruction {
        mnemonic: Mnemonic::DCP,
        addressing_mode: AddressingMode::IndirectX,
    },
    // 0xC4
    Instruction {
        mnemonic: Mnemonic::CPY,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0xC5
    Instruction {
        mnemonic: Mnemonic::CMP,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0xC6
    Instruction {
        mnemonic: Mnemonic::DEC,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0xC7
    Instruction {
        mnemonic: Mnemonic::DCP,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0xC8
    Instruction {
        mnemonic: Mnemonic::INY,
        addressing_mode: AddressingMode::Implied,
    },
    // 0xC9
    Instruction {
        mnemonic: Mnemonic::CMP,
        addressing_mode: AddressingMode::Immediate,
    },
    // 0xCA
    Instruction {
        mnemonic: Mnemonic::DEX,
        addressing_mode: AddressingMode::Implied,
    },
    // 0xCB
    Instruction {
        mnemonic: Mnemonic::Unknown,
        addressing_mode: AddressingMode::Implied,
    },
    // 0xCC
    Instruction {
        mnemonic: Mnemonic::CPY,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0xCD
    Instruction {
        mnemonic: Mnemonic::CMP,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0xCE
    Instruction {
        mnemonic: Mnemonic::DEC,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0xCF
    Instruction {
        mnemonic: Mnemonic::DCP,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0xD0
    Instruction {
        mnemonic: Mnemonic::BNE,
        addressing_mode: AddressingMode::Relative,
    },
    // 0xD1
    Instruction {
        mnemonic: Mnemonic::CMP,
        addressing_mode: AddressingMode::IndirectY,
    },
    // 0xD2
    Instruction {
        mnemonic: Mnemonic::Unknown,
        addressing_mode: AddressingMode::Implied,
    },
    // 0xD3
    Instruction {
        mnemonic: Mnemonic::DCP,
        addressing_mode: AddressingMode::IndirectY,
    },
    // 0xD4
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0xD5
    Instruction {
        mnemonic: Mnemonic::CMP,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0xD6
    Instruction {
        mnemonic: Mnemonic::DEC,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0xD7
    Instruction {
        mnemonic: Mnemonic::DCP,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0xD8
    Instruction {
        mnemonic: Mnemonic::CLD,
        addressing_mode: AddressingMode::Implied,
    },
    // 0xD9
    Instruction {
        mnemonic: Mnemonic::CMP,
        addressing_mode: AddressingMode::AbsoluteY,
    },
    // 0xDA
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::Implied,
    },
    // 0xDB
    Instruction {
        mnemonic: Mnemonic::DCP,
        addressing_mode: AddressingMode::AbsoluteY,
    },
    // 0xDC
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::AbsoluteX,
    },
    // 0xDD
    Instruction {
        mnemonic: Mnemonic::CMP,
        addressing_mode: AddressingMode::AbsoluteX,
    },
    // 0xDE
    Instruction {
        mnemonic: Mnemonic::DEC,
        addressing_mode: AddressingMode::AbsoluteX,
    },
    // 0xDF
    Instruction {
        mnemonic: Mnemonic::DCP,
        addressing_mode: AddressingMode::AbsoluteX,
    },
    // 0xE0
    Instruction {
        mnemonic: Mnemonic::CPX,
        addressing_mode: AddressingMode::Immediate,
    },
    // 0xE1
    Instruction {
        mnemonic: Mnemonic::SBC,
        addressing_mode: AddressingMode::IndirectX,
    },
    // 0xE2
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::Immediate,
    },
    // 0xE3
    Instruction {
        mnemonic: Mnemonic::ISB,
        addressing_mode: AddressingMode::IndirectX,
    },
    // 0xE4
    Instruction {
        mnemonic: Mnemonic::CPX,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0xE5
    Instruction {
        mnemonic: Mnemonic::SBC,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0xE6
    Instruction {
        mnemonic: Mnemonic::INC,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0xE7
    Instruction {
        mnemonic: Mnemonic::ISB,
        addressing_mode: AddressingMode::ZeroPage,
    },
    // 0xE8
    Instruction {
        mnemonic: Mnemonic::INX,
        addressing_mode: AddressingMode::Implied,
    },
    // 0xE9
    Instruction {
        mnemonic: Mnemonic::SBC,
        addressing_mode: AddressingMode::Immediate,
    },
    // 0xEA
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::Implied,
    },
    // 0xEB
    Instruction {
        mnemonic: Mnemonic::SBC,
        addressing_mode: AddressingMode::Immediate,
    },
    // 0xEC
    Instruction {
        mnemonic: Mnemonic::CPX,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0xED
    Instruction {
        mnemonic: Mnemonic::SBC,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0xEE
    Instruction {
        mnemonic: Mnemonic::INC,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0xEF
    Instruction {
        mnemonic: Mnemonic::ISB,
        addressing_mode: AddressingMode::Absolute,
    },
    // 0xF0
    Instruction {
        mnemonic: Mnemonic::BEQ,
        addressing_mode: AddressingMode::Relative,
    },
    // 0xF1
    Instruction {
        mnemonic: Mnemonic::SBC,
        addressing_mode: AddressingMode::IndirectY,
    },
    // 0xF2
    Instruction {
        mnemonic: Mnemonic::Unknown,
        addressing_mode: AddressingMode::Implied,
    },
    // 0xF3
    Instruction {
        mnemonic: Mnemonic::ISB,
        addressing_mode: AddressingMode::IndirectY,
    },
    // 0xF4
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0xF5
    Instruction {
        mnemonic: Mnemonic::SBC,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0xF6
    Instruction {
        mnemonic: Mnemonic::INC,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0xF7
    Instruction {
        mnemonic: Mnemonic::ISB,
        addressing_mode: AddressingMode::ZeroPageX,
    },
    // 0xF8
    Instruction {
        mnemonic: Mnemonic::SED,
        addressing_mode: AddressingMode::Implied,
    },
    // 0xF9
    Instruction {
        mnemonic: Mnemonic::SBC,
        addressing_mode: AddressingMode::AbsoluteY,
    },
    // 0xFA
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::Implied,
    },
    // 0xFB
    Instruction {
        mnemonic: Mnemonic::ISB,
        addressing_mode: AddressingMode::AbsoluteY,
    },
    // 0xFC
    Instruction {
        mnemonic: Mnemonic::NOP,
        addressing_mode: AddressingMode::AbsoluteX,
    },
    // 0xFD
    Instruction {
        mnemonic: Mnemonic::SBC,
        addressing_mode: AddressingMode::AbsoluteX,
    },
    // 0xFE
    Instruction {
        mnemonic: Mnemonic::INC,
        addressing_mode: AddressingMode::AbsoluteX,
    },
    // 0xFF
    Instruction {
        mnemonic: Mnemonic::ISB,
        addressing_mode: AddressingMode::AbsoluteX,
    },
];
