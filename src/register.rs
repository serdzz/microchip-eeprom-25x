#![allow(non_camel_case_types)]

#[allow(dead_code)]
#[repr(u8)]
pub enum Instruction {
    Read = 0b0000_0011,
    Write = 0b0000_0010,
    WriteEnable = 0b0000_0110,
    WriteDisable = 0b0000_0100,
    ReadStatus = 0b0000_01010,
    WriteStatus = 0b0000_01000,
    PageErase = 0b0100_0010,
    SectorErase = 0b1101_1000,
    ChipErase = 0b1100_0111,
    ReleasePowerDown = 0b1010_1011,
    DeepSleepPowerMode = 0b1011_1001
}

#[allow(dead_code)]
#[repr(u8)]
pub enum Erase {
    PageErase = Instruction::PageErase as u8,
    SectorErase = Instruction::SectorErase as u8,
    ChipErase = Instruction::ChipErase as u8,
}