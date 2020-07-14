use bit_field::BitField;

pub struct Status {
    pub value: u8
}

#[allow(dead_code)]
#[repr(u8)]
pub enum WriteProtection {
    None = 0b00,
    Quarter = 0b01,
    Half = 0b10,
    All = 0b11
}

impl Status {
    pub fn write_latch_enabled(&self) -> bool {
        self.value.get_bit(1)
    }

    pub fn write_in_progress(&self) -> bool {
        self.value.get_bit(1)
    }

    pub fn write_protection_level(&self) -> WriteProtection {
        let val = self.value.get_bits(2..3);
        match val {
            0b00 => WriteProtection::None,
            0b01 => WriteProtection::Quarter,
            0b10 => WriteProtection::Half,
            0b11 => WriteProtection::All,
            _ => WriteProtection::None
        }
    }

    pub fn set_write_protection_level(&mut self, protection: WriteProtection) {
        self.value.set_bits(2..3, protection as u8);
    }

    pub fn write_protection_enabled(&mut self) -> bool {
        self.value.get_bit(7)
    }

    pub fn set_write_protection_enabled(&mut self, enabled: bool) {
        self.value.set_bit(7, enabled);
    }
}